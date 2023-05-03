use {
    crate::{
        error::TooLargeSize,
        layout::{Layout, Plain},
    },
    wgpu::{Buffer, Device, Queue, VertexAttribute, VertexStepMode},
};

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
                contents: models.as_bytes(),
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

        queue.write_buffer(&self.buffer, 0, models.as_bytes());
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
#[derive(Copy, Clone)]
pub(crate) struct InstanceModel {
    pub(crate) mat: [[f32; 4]; 4],
}

unsafe impl Plain for InstanceModel {}

impl Layout for InstanceModel {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Instance;
}

impl From<[[f32; 4]; 4]> for InstanceModel {
    fn from(mat: [[f32; 4]; 4]) -> Self {
        Self { mat }
    }
}
