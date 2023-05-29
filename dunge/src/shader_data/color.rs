use crate::layout::Plain;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct ColorUniform([f32; 4]);

impl ColorUniform {
    pub fn new([r, g, b]: [f32; 3]) -> Self {
        Self([r, g, b, 0.])
    }
}

unsafe impl Plain for ColorUniform {}
