use {
    crate::{layout::Plain, shader},
    std::cell::Cell,
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Ambient {
    col: Cell<[f32; 3]>,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Ambient {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let col = [0.; 3];
        let buffer = {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: col.as_bytes(),
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

        Self {
            col: Cell::new(col),
            buffer,
            bind_group,
        }
    }

    pub fn set_ambient(&self, col: [f32; 3], queue: &Queue) {
        if self.col.get() == col {
            return;
        }

        queue.write_buffer(&self.buffer, 0, col.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
