use {
    crate::{
        color::Rgba,
        context::FailedMakeContext,
        draw::Draw,
        format::Format,
        layer::{Layer, SetLayer},
        render::{Input, Render},
        texture::{CopyBuffer, CopyTexture, DrawTexture},
    },
    wgpu::{
        Adapter, Backends, CommandEncoder, Device, Instance, InstanceDescriptor, InstanceFlags,
        Queue, TextureView,
    },
};

pub(crate) struct State {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl State {
    pub async fn new() -> Result<Self, FailedMakeContext> {
        let backends;

        #[cfg(all(target_family = "unix", not(target_os = "macos")))]
        {
            backends = Backends::VULKAN;
        }

        #[cfg(target_family = "windows")]
        {
            backends = Backends::VULKAN;
        }

        #[cfg(target_os = "macos")]
        {
            backends = Backends::METAL;
        }

        #[cfg(target_family = "wasm")]
        {
            backends = Backends::BROWSER_WEBGPU;
        }

        let instance = {
            let desc = InstanceDescriptor {
                backends,
                flags: InstanceFlags::ALLOW_UNDERLYING_NONCOMPLIANT_ADAPTER,
                ..Default::default()
            };

            Instance::new(&desc)
        };

        let adapter = {
            use wgpu::{PowerPreference, RequestAdapterOptions};

            let options = RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
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
            use wgpu::{DeviceDescriptor, Limits};

            let desc = DeviceDescriptor {
                required_limits: Limits {
                    ..if cfg!(target_family = "wasm") {
                        Limits::downlevel_defaults()
                    } else {
                        Limits::default()
                    }
                },
                ..Default::default()
            };

            adapter
                .request_device(&desc)
                .await
                .map_err(FailedMakeContext::RequestDevice)?
        };

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn draw<D>(&self, target: Target, draw: D)
    where
        D: Draw,
    {
        use wgpu::CommandEncoderDescriptor;

        self.queue.submit([]);
        let mut encoder = {
            let desc = CommandEncoderDescriptor::default();
            self.device.create_command_encoder(&desc)
        };

        draw.draw(Frame {
            target,
            encoder: &mut encoder,
        });

        self.queue.submit([encoder.finish()]);
    }
}

pub struct Scheduler<'shed> {
    encoder: &'shed mut CommandEncoder,
}

impl<'shed> Scheduler<'shed> {
    pub fn compute(&'shed mut self) -> Compute<'shed> {
        let desc = wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        };

        let pass = self.encoder.begin_compute_pass(&desc);
        Compute(pass)
    }

    pub fn render<O>(&'shed mut self, target: Target<'_>, opts: O) -> Render<'shed>
    where
        O: Into<Options>,
    {
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

        let pass = self.encoder.begin_render_pass(&desc);
        Render(pass)
    }
}

pub struct Compute<'shed>(#[expect(dead_code)] wgpu::ComputePass<'shed>);

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
    encoder: &'e mut CommandEncoder,
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
        use wgpu::*;

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
        let color_attachment = RenderPassColorAttachment {
            view: self.target.colorv,
            resolve_target: None,
            ops: Operations {
                load: opts
                    .clear_color
                    .map(Rgba::wgpu)
                    .map_or(LoadOp::Load, LoadOp::Clear),
                store: StoreOp::Store,
            },
        };

        let depth_attachment = |view| {
            let ops = Operations {
                load: opts.clear_depth.map_or(LoadOp::Load, LoadOp::Clear),
                store: StoreOp::Store,
            };

            RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(ops),
                stencil_ops: None,
            }
        };

        let desc = RenderPassDescriptor {
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
    colorv: &'v TextureView,
    depthv: Option<&'v TextureView>,
}

impl<'v> Target<'v> {
    pub(crate) fn new(format: Format, colorv: &'v TextureView) -> Self {
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
