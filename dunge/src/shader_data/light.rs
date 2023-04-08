use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

#[derive(Clone, Copy, Default)]
pub struct Source {
    pub pos: [f32; 3],
    pub rad: f32,
    pub col: [f32; 3],
    pub mode: LightMode,
    pub kind: LightKind,
}

#[derive(Clone, Copy, Default)]
pub enum LightMode {
    #[default]
    Smooth,
    Sharp,
}

#[derive(Clone, Copy, Default)]
pub enum LightKind {
    #[default]
    Glow,
    Gloom,
}

pub(crate) struct Light {
    lights_buffer: Buffer,
    n_lights: usize,
    bind_group: BindGroup,
}

impl Light {
    const MAX_N_LIGHTS: usize = 64;

    pub fn new(lights: &[LightModel], device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        if lights.len() > Self::MAX_N_LIGHTS {
            panic!("too many lights");
        }

        let lights_buffer = {
            let default = [LightModel::default()];
            let uniform = if lights.is_empty() { &default } else { lights };
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("lights buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let n_lights_buffer = {
            let len = lights.len() as u32;
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("n lights buffer"),
                contents: len.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::TEXTURED_LIGHTS_BINDING,
                    resource: lights_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::TEXTURED_N_LIGHTS_BINDING,
                    resource: n_lights_buffer.as_entire_binding(),
                },
            ],
            label: Some("lights bind group"),
        });

        Self {
            lights_buffer,
            n_lights: lights.len(),
            bind_group,
        }
    }

    pub fn set_nth_light(&self, n: usize, light: LightModel, queue: &Queue) {
        use std::mem;

        if n >= self.n_lights {
            panic!("wrong light index");
        }

        queue.write_buffer(
            &self.lights_buffer,
            (mem::size_of::<LightModel>() * n) as _,
            light.as_bytes(),
        );
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub(crate) struct LightModel {
    pos: [f32; 3],
    rad: f32,
    col: [f32; 3],
    flags: u32,
}

impl LightModel {
    pub fn new(src: Source) -> Self {
        Self {
            pos: src.pos,
            rad: src.rad,
            col: src.col,
            flags: {
                let sharp = match src.mode {
                    LightMode::Smooth => 0,
                    LightMode::Sharp => 1,
                };

                let gloom = match src.kind {
                    LightKind::Glow => 0,
                    LightKind::Gloom => 1,
                };

                sharp | gloom << 1
            },
        }
    }
}

unsafe impl Plain for LightModel {}
