use wgpu::{Device, Queue};

pub(crate) struct State {
    device: Device,
    queue: Queue,
}

impl State {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}
