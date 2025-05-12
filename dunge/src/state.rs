use {
    crate::{
        buffer::{self, Destination, Format, Size, Source, Texture2d},
        color::{Color, Rgb, Rgba},
        compute::Compute,
        context::FailedMakeContext,
        render::{Render, TargetState},
        runtime::{self, Ticket, Worker},
        usage::u,
    },
    std::sync::Arc,
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
    pub(crate) async fn new() -> Result<Self, FailedMakeContext> {
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
    #[inline]
    pub(crate) fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    #[inline]
    pub(crate) fn device(&self) -> &wgpu::Device {
        &self.device
    }

    #[inline]
    pub(crate) fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    #[inline]
    pub(crate) fn work(&self) {
        self.worker.work();
    }

    #[inline]
    pub(crate) async fn run<F>(&self, f: F)
    where
        F: FnOnce(Scheduler<'_>),
    {
        let mut encoder = {
            let desc = wgpu::CommandEncoderDescriptor::default();
            self.device.create_command_encoder(&desc)
        };

        f(Scheduler(&mut encoder));

        self.queue.submit([encoder.finish()]);

        let ticket = Arc::new(const { Ticket::new() });

        #[cfg(target_family = "wasm")]
        {
            ticket.done();
        }

        #[cfg(not(target_family = "wasm"))]
        {
            // register the callback before starting the work,
            // otherwise the work might complete immediately
            // and the callback would never be called.
            self.queue.on_submitted_work_done({
                let ticket = ticket.clone();
                move || ticket.done()
            });
        }

        self.work();

        ticket.wait().await;
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
        Compute { pass }
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

        Render {
            pass: self.0.begin_render_pass(&desc),
            target: target.state(),
        }
    }

    #[inline]
    pub fn copy<S, D>(&mut self, from: S, to: D)
    where
        S: Source,
        D: Destination,
    {
        if let Err(e) = buffer::try_copy(from, to, self.0) {
            panic!("{e}");
        }
    }

    #[inline]
    pub fn try_copy<S, D>(&mut self, from: S, to: D) -> Result<(), buffer::SizeError>
    where
        S: Source,
        D: Destination,
    {
        buffer::try_copy(from, to, self.0)
    }
}

/// Current layer options.
#[derive(Clone, Copy, Default)]
pub struct Options {
    clear_color: Option<Rgba>,
    clear_depth: Option<f32>,
}

impl Options {
    /// Sets clear color for the layer.
    #[inline]
    pub fn clear_color(mut self, clear: Rgba) -> Self {
        self.clear_color = Some(clear);
        self
    }

    /// Sets clear depth for the layer.
    #[inline]
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

impl From<Rgb> for Options {
    fn from(Color([r, g, b]): Rgb) -> Self {
        Self::default().clear_color(Color([r, g, b, 1.]))
    }
}

/// A target for current frame.
#[derive(Clone, Copy)]
pub struct Target<'view> {
    format: Format,
    colorv: &'view wgpu::TextureView,
    depthv: Option<&'view wgpu::TextureView>,
}

impl<'view> Target<'view> {
    #[inline]
    pub(crate) fn new(format: Format, colorv: &'view wgpu::TextureView) -> Self {
        Self {
            format,
            colorv,
            depthv: None,
        }
    }

    #[inline]
    pub(crate) fn state(&self) -> TargetState {
        TargetState {
            format: self.format,
            use_depth: self.depthv.is_some(),
        }
    }
}

/// Something that contains a [target](Target).
pub trait AsTarget {
    fn as_target(&self) -> Target<'_>;
}

impl<A> AsTarget for &A
where
    A: AsTarget,
{
    #[inline]
    fn as_target(&self) -> Target<'_> {
        (**self).as_target()
    }
}

impl<U> AsTarget for Texture2d<U>
where
    U: u::Render,
{
    #[inline]
    fn as_target(&self) -> Target<'_> {
        Target::new(self.format(), self.view())
    }
}

impl<U, V> AsTarget for RenderBuffer<U, V>
where
    U: u::Render,
    V: u::Render,
{
    #[inline]
    fn as_target(&self) -> Target<'_> {
        let mut target = self.color.as_target();
        target.depthv = Some(self.depth.view());
        target
    }
}

/// Pair of color and depth buffer.
pub struct RenderBuffer<U, V> {
    color: Texture2d<U>,
    depth: Texture2d<V>,
}

impl<U, V> RenderBuffer<U, V> {
    pub fn new(color: Texture2d<U>, depth: Texture2d<V>) -> Self
    where
        U: u::Render,
        V: u::Render,
    {
        assert_eq!(
            depth.format(),
            Format::Depth,
            "the depth texture must have the depth format",
        );

        assert_eq!(
            color.size(),
            depth.size(),
            "color and depth textures must be the same size",
        );

        Self { color, depth }
    }

    #[inline]
    pub fn size(&self) -> Size {
        self.color.size()
    }

    #[inline]
    pub fn color(&self) -> &Texture2d<U> {
        &self.color
    }

    #[inline]
    pub fn depth(&self) -> &Texture2d<V> {
        &self.depth
    }
}
