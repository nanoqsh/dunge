use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct AmbientUniform([f32; 4]);

impl AmbientUniform {
    pub fn new([r, g, b]: [f32; 3]) -> Self {
        Self([r, g, b, 0.])
    }
}
