use {
    crate::{
        color::Rgba,
        context::Error,
        draw::Draw,
        format::Format,
        layer::{Layer, SetLayer},
        texture::{CopyBuffer, CopyTexture},
    },
    std::sync::atomic::{self, AtomicUsize},
    wgpu::{Color, CommandEncoder, Device, Instance, LoadOp, Queue, TextureView},
};

#[cfg(feature = "winit")]
use wgpu::Adapter;

pub(crate) struct State {
    #[cfg(feature = "winit")]
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader_ids: AtomicUsize,
}

impl State {
    pub async fn new(instance: &Instance) -> Result<Self, Error> {
        let adapter = {
            use wgpu::{PowerPreference, RequestAdapterOptions};

            let options = RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            };

            instance
                .request_adapter(&options)
                .await
                .ok_or(Error::BackendSelection)?
        };

        let backend = adapter.get_info().backend;
        log::info!("selected backend: {backend:?}");

        let (device, queue) = {
            use wgpu::{DeviceDescriptor, Limits};

            let desc = DeviceDescriptor {
                required_limits: Limits {
                    ..if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    }
                },
                ..Default::default()
            };

            adapter
                .request_device(&desc, None)
                .await
                .map_err(Error::RequestDevice)?
        };

        Ok(Self {
            #[cfg(feature = "winit")]
            adapter,
            device,
            queue,
            shader_ids: AtomicUsize::default(),
        })
    }

    #[cfg(feature = "winit")]
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn next_shader_id(&self) -> usize {
        self.shader_ids.fetch_add(1, atomic::Ordering::Relaxed)
    }

    pub fn draw<D>(&self, view: RenderView, draw: D)
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
            view,
            encoder: &mut encoder,
        });

        self.queue.submit([encoder.finish()]);
    }
}

#[derive(Clone, Copy, Default)]
pub struct Options {
    clear: Option<Rgba>,
}

impl Options {
    pub fn with_clear(mut self, clear: Rgba) -> Self {
        self.clear = Some(clear);
        self
    }

    fn clear(self) -> LoadOp<Color> {
        self.clear.map_or(LoadOp::Load, |col| {
            let [r, g, b, a] = col.0.map(f64::from);
            LoadOp::Clear(Color { r, g, b, a })
        })
    }
}

impl From<Rgba> for Options {
    fn from(v: Rgba) -> Self {
        Self::default().with_clear(v)
    }
}

pub struct Frame<'v, 'e> {
    view: RenderView<'v>,
    encoder: &'e mut CommandEncoder,
}

impl Frame<'_, '_> {
    pub fn layer<'p, V, I, O>(&'p mut self, layer: &'p Layer<V, I>, opts: O) -> SetLayer<'p, V, I>
    where
        O: Into<Options>,
    {
        use wgpu::*;

        assert!(
            self.view.format == layer.format(),
            "layer format doesn't match frame format",
        );

        let opts = opts.into();
        let attachment = RenderPassColorAttachment {
            view: self.view.txview,
            resolve_target: None,
            ops: Operations {
                load: opts.clear(),
                store: StoreOp::Store,
            },
        };

        let desc = RenderPassDescriptor {
            color_attachments: &[Some(attachment)],
            ..Default::default()
        };

        let pass = self.encoder.begin_render_pass(&desc);
        layer.set(pass)
    }

    pub fn copy_texture<T>(&mut self, buffer: &CopyBuffer, texture: &T)
    where
        T: CopyTexture,
    {
        buffer.copy_texture(texture.copy_texture(), self.encoder);
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RenderView<'v> {
    txview: &'v TextureView,
    format: Format,
}

impl<'v> RenderView<'v> {
    pub fn new(txview: &'v TextureView, format: Format) -> Self {
        Self { txview, format }
    }
}
