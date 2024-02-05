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
    let compute = || {
        let m = -sl::mat2(sl::vec2(1., 0.), sl::vec2(0., 1.));
        let [m0, m1, m3] = sl::thunk(m);
        let v = m0.x() + (-m1.y());
        let z = sl::splat_vec3(1.).z();

        Out {
            place: sl::vec4_concat(m3.x(), v) * sl::f32(1) * z,
            color: sl::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    fs::write("tests/shader.wgsl", shader.debug_wgsl())?;
    Ok(())
}
