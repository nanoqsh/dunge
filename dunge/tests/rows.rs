#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use dunge::{
        glam::{Mat4, Vec4},
        instance::Row,
        prelude::*,
        sl::{self, PassInstance, Render},
    };

    #[derive(Instance)]
    struct Transform {
        f: Row<f32>,
        v: Row<Vec4>,
        m: Row<Mat4>,
    }

    let code = |PassInstance(t): PassInstance<Transform>| Render {
        place: t.m * t.v * t.f,
        color: sl::vec4_splat(1.),
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rows.wgsl"));

    Ok(())
}
