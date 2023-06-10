use {
    crate::{layout::Plain, postproc::PostProcessor},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct PostShaderData {
    bind_group: BindGroup,
    buffer: Buffer,
}

impl PostShaderData {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let buffer = {
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
                binding: PostProcessor::DATA_BINDING,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("post bind group"),
        });

        Self { bind_group, buffer }
    }

    pub fn resize(&self, size: [f32; 2], factor: [f32; 2], queue: &Queue) {
        let uniform = PostShaderDataUniform::new(size, factor);
        queue.write_buffer(&self.buffer, 0, uniform.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct PostShaderDataUniform {
    size: [f32; 2],
    factor: [f32; 2],
}

impl PostShaderDataUniform {
    fn new(size: [f32; 2], factor: [f32; 2]) -> Self {
        Self { size, factor }
    }
}

unsafe impl Plain for PostShaderDataUniform {}
