use {
    crate::error::TooLargeSize,
    bytemuck::{Pod, Zeroable},
    wgpu::{Buffer, Device, Queue, VertexBufferLayout, VertexStepMode},
};

type Mat = [[f32; 4]; 4];

pub(crate) struct Instance {
    buffer: Buffer,
    n_instances: u32,
}

impl Instance {
    pub fn new(models: &[InstanceModel], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(models),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
            n_instances: models.len().try_into().expect("convert instances len"),
        }
    }

    pub fn update_models(
        &self,
        models: &[InstanceModel],
        queue: &Queue,
    ) -> Result<(), TooLargeSize> {
        if self.n_instances as usize != models.len() {
            return Err(TooLargeSize);
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(models));
        Ok(())
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn n_instances(&self) -> u32 {
        self.n_instances
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct InstanceModel {
    pub(crate) mat: Mat,
}

impl InstanceModel {
    pub const LOCATION_OFFSET: u32 = 4;
    pub const LAYOUT: VertexBufferLayout<'_> = {
        use std::mem;

        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x4,
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
            ],
        }
    };
}

impl From<Mat> for InstanceModel {
    fn from(mat: Mat) -> Self {
        Self { mat }
    }
}
