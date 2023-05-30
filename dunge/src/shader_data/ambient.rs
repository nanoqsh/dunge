use crate::layout::Plain;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct AmbientUniform([f32; 4]);

impl AmbientUniform {
    pub fn new([r, g, b]: [f32; 3]) -> Self {
        Self([r, g, b, 0.])
    }
}

unsafe impl Plain for AmbientUniform {}
