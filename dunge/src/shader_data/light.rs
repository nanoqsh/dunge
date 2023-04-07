use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Light {
    light_buffer: Buffer,
    ambient_buffer: Buffer,
    bind_group: BindGroup,
}

impl Light {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let light_buffer = {
            let uniform = LightUniform::default();
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("light data buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let ambient_buffer = {
            let uniform: [f32; 3] = [0.; 3];
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("light data buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::TEXTURED_LIGHT_BINDING,
                    resource: light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::TEXTURED_AMBIENT_BINDING,
                    resource: ambient_buffer.as_entire_binding(),
                },
            ],
            label: Some("light bind group"),
        });

        Self {
            light_buffer,
            ambient_buffer,
            bind_group,
        }
    }

    pub fn set_light(&self, pos: [f32; 3], rad: f32, col: [f32; 3], queue: &Queue) {
        let uniform = LightUniform::new(pos, rad, col, false, false);
        queue.write_buffer(&self.light_buffer, 0, uniform.as_bytes());
    }

    pub fn set_ambient(&self, ambient: [f32; 3], queue: &Queue) {
        queue.write_buffer(&self.ambient_buffer, 0, ambient.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct LightUniform {
    pos: [f32; 3],
    rad: f32,
    col: [f32; 3],
    flags: u32,
}

impl LightUniform {
    fn new(pos: [f32; 3], rad: f32, col: [f32; 3], sharp: bool, shadow: bool) -> Self {
        Self {
            pos,
            rad,
            col,
            flags: (sharp as u32) | ((shadow as u32) << 1),
        }
    }
}

unsafe impl Plain for LightUniform {}
