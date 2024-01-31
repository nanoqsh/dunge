#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        glam::Vec4,
        sl::{self, Out},
    },
    std::{error, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    let triangle = || Out {
        place: sl::splat_vec4(1.) * sl::f32(1) * 1.,
        color: sl::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    fs::write("tests/shader.wgsl", shader.debug_wgsl())?;
    Ok(())
}
