use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
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
