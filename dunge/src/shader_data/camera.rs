use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(view_proj: [[f32; 4]; 4]) -> Self {
        Self { view_proj }
    }

    const IDENTITY: [[f32; 4]; 4] = [
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_proj: Self::IDENTITY,
        }
    }
}
