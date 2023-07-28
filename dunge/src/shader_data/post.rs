use {
    crate::{framebuffer::BufferSize, postproc::PostProcessor},
    bytemuck::{Pod, Zeroable},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct PostShaderData {
    bind_group: BindGroup,
    buf: Buffer,
}

impl PostShaderData {
    pub fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let buf = {
            let uniform = PostShaderDataUniform::new((1., 1.), (1., 1.));
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("post data buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: PostProcessor::DATA_BINDING,
                resource: buf.as_entire_binding(),
            }],
            label: Some("post bind group"),
        });

        Self { bind_group, buf }
    }

    pub fn resize(&self, size: BufferSize, factor: (f32, f32), queue: &Queue) {
        let uniform = PostShaderDataUniform::new(size.into(), factor);
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct PostShaderDataUniform {
    size: [f32; 2],
    step: [f32; 2],
    factor: [f32; 2],
    pad: u64,
}

impl PostShaderDataUniform {
    fn new(size: (f32, f32), factor: (f32, f32)) -> Self {
        const STEP_FACTOR: f32 = 0.5;

        let size = size.into();
        Self {
            size,
            step: size.map(|v| STEP_FACTOR / v),
            factor: factor.into(),
            pad: 0,
        }
    }
}
