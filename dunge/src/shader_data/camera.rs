use crate::layout::Plain;

#[repr(C)]
#[derive(Copy, Clone)]
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

unsafe impl Plain for CameraUniform {}
