use {
    crate::{
        draw::Draw,
        layer::{Layer, SetLayer},
        texture::{CopyBuffer, CopyTexture},
    },
    wgpu::{Adapter, CommandEncoder, Device, Instance, Queue, TextureView},
};

pub(crate) struct State {
    adapter: Adapter,
    device: Device,
    queue: Queue,
    encoders: Encoders,
}

impl State {
    pub async fn new(instance: &Instance) -> Self {
        use wgpu::*;

        let adapter = {
            let options = RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            };

            instance.request_adapter(&options).await.unwrap()
        };

        let backend = adapter.get_info().backend;
        println!("backend: {backend:?}");

        let (device, queue) = {
            let desc = DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
            };

            adapter.request_device(&desc, None).await.unwrap()
        };

        Self {
            adapter,
            device,
            queue,
            encoders: Encoders::default(),
        }
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

    pub fn render<D>(&mut self, view: &TextureView, draw: D)
    where
        D: Draw,
    {
        let frame = self.encoders.make(&self.device, view);
        draw.draw(frame);

        let buffers = self.encoders.drain().map(CommandEncoder::finish);
        self.queue.submit(buffers);
    }
}

pub struct Frame<'v, 'e> {
    view: &'v TextureView,
    device: &'e Device,
    encoders: &'e mut Encoders,
    id: usize,
}

impl Frame<'_, '_> {
    pub fn subframe<'e, 'v>(&'e mut self, view: &'v TextureView) -> Frame<'v, 'e> {
        self.encoders.make(self.device, view)
    }

    pub fn layer<'p, V>(&'p mut self, layer: &'p Layer<V>) -> SetLayer<'p, V> {
        use wgpu::*;

        let clear = Color {
            r: 0.19,
            g: 0.06,
            b: 0.12,
            a: 1.,
        };

        let attachment = RenderPassColorAttachment {
            view: self.view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(clear),
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
    fn make<'e, 'v>(&'e mut self, device: &'e Device, view: &'v TextureView) -> Frame<'v, 'e> {
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
