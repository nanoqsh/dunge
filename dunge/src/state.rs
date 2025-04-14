use {
    crate::{
        color::Rgba,
        context::FailedMakeContext,
        draw::Draw,
        format::Format,
        layer::{Layer, SetLayer},
        render::{Input, Render},
        runtime::{self, Worker},
        texture::{CopyBuffer, CopyTexture, DrawTexture},
    },
    std::{
        future,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        task::{Poll, Waker},
    },
};

const DEFAULT_BACKEND: wgpu::Backends = {
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        wgpu::Backends::VULKAN
    }

    #[cfg(target_family = "windows")]
    {
        wgpu::Backends::VULKAN
    }

    #[cfg(target_os = "macos")]
    {
        wgpu::Backends::METAL
    }

    #[cfg(target_family = "wasm")]
    {
        wgpu::Backends::BROWSER_WEBGPU
    }
};

pub(crate) struct State {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    worker: Worker,
}

impl State {
    pub async fn new() -> Result<Self, FailedMakeContext> {
        let instance = {
            let desc = wgpu::InstanceDescriptor {
                backends: DEFAULT_BACKEND,
                flags: wgpu::InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                ..Default::default()
            };

            wgpu::Instance::new(&desc)
        };

        let adapter = {
            let options = wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            };

            instance
                .request_adapter(&options)
                .await
                .map_err(FailedMakeContext::BackendSelection)?
        };

        let backend = adapter.get_info().backend;
        log::info!("selected backend: {backend:?}");

        let (device, queue) = {
            let desc = wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits {
                    ..if cfg!(target_family = "wasm") {
                        wgpu::Limits::downlevel_defaults()
                    } else {
                        wgpu::Limits::default()
                    }
                },
                ..Default::default()
            };

            adapter
                .request_device(&desc)
                .await
                .map_err(FailedMakeContext::RequestDevice)?
        };

        let worker = runtime::poll_in_background(instance.clone());

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            worker,
        })
    }

    #[allow(dead_code)]
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    #[allow(dead_code)]
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn draw<D>(&self, target: Target, draw: D)
    where
        D: Draw,
    {
        self.queue.submit([]);
        let mut encoder = {
            let desc = wgpu::CommandEncoderDescriptor::default();
            self.device.create_command_encoder(&desc)
        };

        draw.draw(Frame {
            target,
            encoder: &mut encoder,
        });

        self.queue.submit([encoder.finish()]);
    }

    pub async fn run<F>(&self, f: F)
    where
        F: FnOnce(Scheduler<'_>),
    {
        let mut encoder = {
            let desc = wgpu::CommandEncoderDescriptor::default();
            self.device.create_command_encoder(&desc)
        };

        f(Scheduler(&mut encoder));

        self.queue.submit([encoder.finish()]);
        self.worker.work();

        struct Notify {
            done: AtomicBool,
            waker: Mutex<Waker>,
        }

        let notify = Arc::new(Notify {
            done: AtomicBool::new(false),
            waker: Mutex::new(Waker::noop().clone()),
        });

        self.queue.on_submitted_work_done({
            let notify = notify.clone();
            move || {
                notify.done.store(true, Ordering::Release);
                notify.waker.lock().expect("lock waker").wake_by_ref();
            }
        });

        let fu = future::poll_fn(|cx| {
            if notify.done.load(Ordering::Acquire) {
                Poll::Ready(())
            } else {
                *notify.waker.lock().expect("lock waker") = cx.waker().clone();
                Poll::Pending
            }
        });

        fu.await;
    }
}

pub struct Scheduler<'shed>(&'shed mut wgpu::CommandEncoder);

impl Scheduler<'_> {
    #[inline]
    pub fn compute(&mut self) -> Compute<'_> {
        let desc = wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        };

        let pass = self.0.begin_compute_pass(&desc);
        Compute(pass)
    }

    #[inline]
    pub fn render<T, O>(&mut self, target: T, opts: O) -> Render<'_>
    where
        T: AsTarget,
        O: Into<Options>,
    {
        let target = target.as_target();
        let opts = opts.into();

        let color_attachment = wgpu::RenderPassColorAttachment {
            view: target.colorv,
            resolve_target: None,
            ops: wgpu::Operations {
                load: opts
                    .clear_color
                    .map(Rgba::wgpu)
                    .map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                store: wgpu::StoreOp::Store,
            },
        };

        let depth_attachment = |view| {
            let ops = wgpu::Operations {
                load: opts
                    .clear_depth
                    .map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                store: wgpu::StoreOp::Store,
            };

            wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(ops),
                stencil_ops: None,
            }
        };

        let desc = wgpu::RenderPassDescriptor {
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: target.depthv.map(depth_attachment),
            ..Default::default()
        };

        let pass = self.0.begin_render_pass(&desc);
        Render(pass)
    }

    #[inline]
    pub fn copy(&self, _from: (), _to: ()) {
        todo!()
    }
}

pub struct Compute<'com>(#[expect(dead_code)] wgpu::ComputePass<'com>);

/// Current layer options.
#[derive(Clone, Copy, Default)]
pub struct Options {
    clear_color: Option<Rgba>,
    clear_depth: Option<f32>,
}

impl Options {
    /// Sets clear color for the layer.
    pub fn clear_color(mut self, clear: Rgba) -> Self {
        self.clear_color = Some(clear);
        self
    }

    /// Sets clear depth for the layer.
    pub fn clear_depth(mut self, clear: f32) -> Self {
        self.clear_depth = Some(clear);
        self
    }
}

impl From<Rgba> for Options {
    fn from(v: Rgba) -> Self {
        Self::default().clear_color(v)
    }
}

/// The frame type for drawing and copying operations.
pub struct Frame<'v, 'e> {
    target: Target<'v>,
    encoder: &'e mut wgpu::CommandEncoder,
}

impl Frame<'_, '_> {
    pub fn set_layer<'p, V, I, S, O>(
        &'p mut self,
        layer: &'p Layer<Input<V, I, S>>,
        opts: O,
    ) -> SetLayer<'p, (V, I), S>
    where
        O: Into<Options>,
    {
        assert_eq!(
            self.target.format,
            layer.format(),
            "layer format doesn't match frame format",
        );

        assert!(
            !layer.depth() || self.target.depthv.is_some(),
            "the target for a layer with depth must contain a depth buffer",
        );

        let opts = opts.into();
        let color_attachment = wgpu::RenderPassColorAttachment {
            view: self.target.colorv,
            resolve_target: None,
            ops: wgpu::Operations {
                load: opts
                    .clear_color
                    .map(Rgba::wgpu)
                    .map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                store: wgpu::StoreOp::Store,
            },
        };

        let depth_attachment = |view| {
            let ops = wgpu::Operations {
                load: opts
                    .clear_depth
                    .map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                store: wgpu::StoreOp::Store,
            };

            wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(ops),
                stencil_ops: None,
            }
        };

        let desc = wgpu::RenderPassDescriptor {
            color_attachments: &[Some(color_attachment)],
            depth_stencil_attachment: self.target.depthv.map(depth_attachment),
            ..Default::default()
        };

        let pass = self.encoder.begin_render_pass(&desc);
        layer._set(pass)
    }

    pub fn copy_texture<T>(&mut self, buffer: &CopyBuffer, texture: &T)
    where
        T: CopyTexture,
    {
        buffer.copy_texture(texture.copy_texture(), self.encoder);
    }
}

/// A target for current frame.
#[derive(Clone, Copy)]
pub struct Target<'v> {
    format: Format,
    colorv: &'v wgpu::TextureView,
    depthv: Option<&'v wgpu::TextureView>,
}

impl<'v> Target<'v> {
    pub(crate) fn new(format: Format, colorv: &'v wgpu::TextureView) -> Self {
        Self {
            format,
            colorv,
            depthv: None,
        }
    }
}

/// Something that contains a [target](Target).
pub trait AsTarget {
    fn as_target(&self) -> Target;
}

impl<T> AsTarget for T
where
    T: DrawTexture,
{
    fn as_target(&self) -> Target {
        let texture = self.draw_texture();
        Target::new(texture.format(), texture.view())
    }
}

impl<T, D> AsTarget for RenderBuffer<T, D>
where
    T: DrawTexture,
    D: DrawTexture,
{
    fn as_target(&self) -> Target {
        let mut target = self.color.as_target();
        target.depthv = Some(self.depth.draw_texture().view());
        target
    }
}

/// Pair of color and depth buffer.
#[derive(Clone, Copy)]
pub struct RenderBuffer<T, D> {
    color: T,
    depth: D,
}

impl<T, D> RenderBuffer<T, D> {
    pub fn new(color: T, depth: D) -> Self
    where
        T: DrawTexture,
        D: DrawTexture,
    {
        let color_texture = color.draw_texture();
        let depth_texture = depth.draw_texture();
        assert_eq!(
            depth_texture.format(),
            Format::Depth,
            "the depth texture must have the depth format",
        );

        assert_eq!(
            color_texture.size(),
            depth_texture.size(),
            "color and depth textures must be the same size",
        );

        Self { color, depth }
    }

    pub fn size(&self) -> (u32, u32)
    where
        T: DrawTexture,
    {
        self.color.draw_texture().size()
    }

    pub fn color(&self) -> &T {
        &self.color
    }

    pub fn depth(&self) -> &D {
        &self.depth
    }
}
