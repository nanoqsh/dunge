use {
    crate::{layout::Plain, shader_consts},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Screen {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Screen {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            *,
        };

        let uniform = ScreenUniform { size: [1., 1.] };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("screen buffer"),
            contents: uniform.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: shader_consts::post::SCREEN.binding,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("screen bind group"),
        });

        Self { buffer, bind_group }
    }

    pub(crate) fn resize(&self, (width, height): (u32, u32), queue: &Queue) {
        let data = ScreenUniform {
            size: [width as f32, height as f32],
        };

        queue.write_buffer(&self.buffer, 0, data.as_bytes());
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct ScreenUniform {
    size: [f32; 2],
}

unsafe impl Plain for ScreenUniform {}
