use {
    crate::layout::{InstanceModel, Plain},
    wgpu::{Buffer, Device, Queue},
};

pub(crate) struct Instance {
    buffer: Buffer,
    n_instances: u32,
}

impl Instance {
    pub(crate) fn new(models: &[InstanceModel], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: models.as_bytes(),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let n_instances = models.len().try_into().expect("convert instances len");

        Self {
            buffer,
            n_instances,
        }
    }

    pub(crate) fn update_models(&mut self, models: &[InstanceModel], queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, models.as_bytes());
        self.n_instances = models.len().try_into().expect("convert instances len");
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(crate) fn n_instances(&self) -> u32 {
        self.n_instances
    }
}
