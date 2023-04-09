use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Ambient {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Ambient {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let buffer = {
            let uniform: [f32; 3] = [0.; 3];
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: shader::AMBIENT_BINDING,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("ambient bind group"),
        });

        Self { buffer, bind_group }
    }

    pub fn set_ambient(&self, ambient: [f32; 3], queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, ambient.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
