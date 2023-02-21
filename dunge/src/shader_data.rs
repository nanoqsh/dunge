use {
    crate::{layout::Plain, shader},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct PostShaderData {
    buffer_data: Buffer,
    buffer_vignette: Buffer,
    bind_group: BindGroup,
}

impl PostShaderData {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let buffer_data = {
            let uniform = PostShaderDataUniform::new(1., 1.);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("post data buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let buffer_vignette = {
            let uniform = PostShaderVignetteUniform::new([0.; 4]);
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("post vignette buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                BindGroupEntry {
                    binding: shader::POST_DATA_BINDING,
                    resource: buffer_data.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: shader::POST_VIGNETTE_BINDING,
                    resource: buffer_vignette.as_entire_binding(),
                },
            ],
            label: Some("post vignette bind group"),
        });

        Self {
            buffer_data,
            buffer_vignette,
            bind_group,
        }
    }

    pub(crate) fn resize(&self, (width, height): (u32, u32), queue: &Queue) {
        let uniform = PostShaderDataUniform::new(width as f32, height as f32);
        queue.write_buffer(&self.buffer_data, 0, uniform.as_bytes());
    }

    pub(crate) fn set_vignette_color(&self, col: [f32; 4], queue: &Queue) {
        let uniform = PostShaderVignetteUniform::new(col);
        queue.write_buffer(&self.buffer_vignette, 0, uniform.as_bytes());
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

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct PostShaderVignetteUniform {
    col: [f32; 4],
}

impl PostShaderVignetteUniform {
    fn new(col: [f32; 4]) -> Self {
        Self { col }
    }
}

unsafe impl Plain for PostShaderVignetteUniform {}
