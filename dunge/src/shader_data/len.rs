use crate::layout::Plain;

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct LenUniform([u32; 4]);

impl LenUniform {
    pub fn new(n: u32) -> Self {
        Self([n, 0, 0, 0])
    }

    pub fn get(self) -> u32 {
        let Self([n, ..]) = self;
        n
    }
}

unsafe impl Plain for LenUniform {}
