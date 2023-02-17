use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct PostShaderData {
    buffer: Buffer,
    bind_group: BindGroup,
}

impl PostShaderData {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let uniform = PostShaderDataUniform::new(1., 1.);

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("post data buffer"),
            contents: uniform.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: shader::POST_DATA_BINDING,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("post data bind group"),
        });

        Self { buffer, bind_group }
    }

    pub(crate) fn resize(&self, (width, height): (u32, u32), queue: &Queue) {
        let data = PostShaderDataUniform::new(width as f32, height as f32);
        queue.write_buffer(&self.buffer, 0, data.as_bytes());
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct PostShaderDataUniform {
    size: [f32; 2],
    _pad: [f32; 2],
}

impl PostShaderDataUniform {
    fn new(width: f32, height: f32) -> Self {
        Self {
            size: [width, height],
            _pad: [0.; 2],
        }
    }
}

unsafe impl Plain for PostShaderDataUniform {}
