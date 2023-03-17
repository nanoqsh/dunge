use {
    crate::layout::{InstanceModel, Plain},
    wgpu::{Buffer, Device, Queue},
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

    pub fn update_models(&mut self, models: &[InstanceModel], queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, models.as_bytes());
        self.n_instances = models.len().try_into().expect("convert instances len");
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn n_instances(&self) -> u32 {
        self.n_instances
    }
}
