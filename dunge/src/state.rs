use {
    crate::{
        color::Rgba,
        context::Error,
        draw::Draw,
        layer::{Layer, SetLayer},
        texture::{CopyBuffer, CopyTexture, Format, Texture},
    },
    std::sync::atomic::{self, AtomicUsize},
    wgpu::{Adapter, Color, CommandEncoder, Device, Instance, LoadOp, Queue, TextureView},
};

pub(crate) struct State {
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

        let (device, queue) = {
            use wgpu::{DeviceDescriptor, Limits};

            let desc = DeviceDescriptor {
                limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                ..Default::default()
            };

            adapter
                .request_device(&desc, None)
                .await
                .map_err(|_| Error::RequestDevice)?
        };

        Ok(Self {
            adapter,
            device,
            queue,
            shader_ids: AtomicUsize::default(),
        })
    }

    #[allow(dead_code)]
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

    pub fn draw<V, D>(&self, render: &mut Render, view: &V, draw: D)
    where
        V: AsView,
        D: Draw,
    {
        let frame = render.encoders.make(&self.device, view.as_view());
        draw.draw(frame);

        let buffers = render.encoders.drain().map(CommandEncoder::finish);
        self.queue.submit(buffers);
    }
}

#[derive(Default)]
pub struct Render {
    encoders: Encoders,
}

#[derive(Clone, Copy, Default)]
pub struct Options {
    clear_color: Option<Rgba>,
}

impl Options {
    fn clear_color(self) -> LoadOp<Color> {
        self.clear_color.map_or(LoadOp::Load, |col| {
            let [r, g, b, a] = col.0.map(f64::from);
            LoadOp::Clear(Color { r, g, b, a })
        })
    }
}

pub struct Frame<'v, 'e> {
    view: View<'v>,
    device: &'e Device,
    encoders: &'e mut Encoders,
    id: usize,
}

impl Frame<'_, '_> {
    pub fn subframe<'e, 'v, V>(&'e mut self, view: &'v V) -> Frame<'v, 'e>
    where
        V: AsView,
    {
        self.encoders.make(self.device, view.as_view())
    }

    pub fn layer<'p, V>(&'p mut self, layer: &'p Layer<V>, opts: Options) -> SetLayer<'p, V> {
        use wgpu::*;

        if self.view.format != layer.format() {
            panic!("layer format doesn't match frame format");
        }

        let attachment = RenderPassColorAttachment {
            view: self.view.txview,
            resolve_target: None,
            ops: Operations {
                load: opts.clear_color(),
                store: StoreOp::Store,
            },
        };

        let desc = RenderPassDescriptor {
            color_attachments: &[Some(attachment)],
            ..Default::default()
        };

        let encoder = self.encoders.get_mut(self.id);
        let pass = encoder.begin_render_pass(&desc);
        layer.set(pass)
    }

    pub fn copy_texture<T>(&mut self, buffer: &CopyBuffer, texture: &T)
    where
        T: CopyTexture,
    {
        let encoder = self.encoders.get_mut(self.id);
        buffer.copy_texture(texture.copy_texture(), encoder);
    }
}

#[derive(Default)]
struct Encoders(Vec<CommandEncoder>);

impl Encoders {
    fn make<'e, 'v>(&'e mut self, device: &'e Device, view: View<'v>) -> Frame<'v, 'e> {
        use wgpu::CommandEncoderDescriptor;

        let encoder = {
            let desc = CommandEncoderDescriptor::default();
            device.create_command_encoder(&desc)
        };

        let id = self.0.len();
        self.0.push(encoder);
        Frame {
            view,
            device,
            encoders: self,
            id,
        }
    }

    fn get_mut(&mut self, id: usize) -> &mut CommandEncoder {
        &mut self.0[id]
    }

    fn drain(&mut self) -> impl Iterator<Item = CommandEncoder> + '_ {
        self.0.drain(..)
    }
}

pub trait AsView {
    fn as_view(&self) -> View;
}

#[derive(Clone, Copy)]
pub struct View<'v> {
    txview: &'v TextureView,
    format: Format,
}

impl AsView for Texture {
    fn as_view(&self) -> View {
        View {
            txview: self.view(),
            format: self.format(),
        }
    }
}
