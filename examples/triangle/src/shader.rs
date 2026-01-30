use {dunge::store::Uniform, dunge_winit::prelude::*, glam::Vec4, std::f32::consts};

#[derive(Clone, Copy, Value)]
pub struct Index {
    #[index]
    pub index: u32,
}

#[dunge(vertex)]
pub fn vs(ind: Index, offset: Uniform<f32>) -> Vec4 {
    let third = const { consts::TAU / 3. };
    let i = ind.index as f32 * third + offset.read();
    Vec4::new(sl::cos(i), sl::sin(i), 0., 1.)
}

#[dunge(fragment)]
pub fn fs() -> Vec4 {
    Vec4::new(1., 0.4, 0.8, 1.)
}
