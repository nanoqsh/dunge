#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn shader_calc() -> Result<(), Error> {
    use dunge::{
        glam::Vec4,
        sl::{self, Out},
    };

    let compute = || {
        let m = -sl::mat2(sl::vec2(1., 0.), sl::vec2(0., 1.));
        let [m0, m1, m3] = sl::thunk(m);
        let v = m0.x() + m1.y();
        let z = sl::splat_vec3(1.).z();

        Out {
            place: sl::vec4_concat(m3.x(), v) * sl::f32(1) * z,
            color: sl::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    assert_eq!(shader.debug_wgsl(), include_str!("shader_calc.wgsl"));
    Ok(())
}

#[test]
fn shader_if() -> Result<(), Error> {
    use dunge::{
        glam::Vec4,
        sl::{self, Out},
    };

    let compute = || {
        let a = Vec4::splat(3.);
        let b = sl::splat_vec4(2.) * 2.;
        let x = sl::if_then_else(true, a, b);

        Out {
            place: x,
            color: sl::splat_vec4(1.),
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    assert_eq!(shader.debug_wgsl(), include_str!("shader_if.wgsl"));
    Ok(())
}
