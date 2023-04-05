use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct PostShaderData {
    buffer_data: Buffer,
    bind_group: BindGroup,
}

impl PostShaderData {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let buffer_data = {
            let uniform = PostShaderDataUniform::new([1., 1.], [1., 1.]);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("post data buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: shader::POST_DATA_BINDING,
                resource: buffer_data.as_entire_binding(),
            }],
            label: Some("post bind group"),
        });

        Self {
            buffer_data,
            bind_group,
        }
    }

    pub fn resize(&self, size: [f32; 2], factor: [f32; 2], queue: &Queue) {
        let uniform = PostShaderDataUniform::new(size, factor);
        queue.write_buffer(&self.buffer_data, 0, uniform.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct PostShaderDataUniform {
    size: [f32; 2],
    factor: [f32; 2],
}

impl PostShaderDataUniform {
    fn new(size: [f32; 2], factor: [f32; 2]) -> Self {
        Self { size, factor }
    }
}

unsafe impl Plain for PostShaderDataUniform {}
