#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            instance::RowOld,
            prelude::*,
            sl_old::{self, PassInstance, Render},
        },
        glam::{Mat4, Vec4},
    };

    #[derive(Instance)]
    struct Transform {
        f: RowOld<f32>,
        v: RowOld<Vec4>,
        m: RowOld<Mat4>,
    }

    let code = |PassInstance(t): PassInstance<Transform>| Render {
        place: t.m * t.v * t.f,
        color: sl_old::vec4_splat(1.),
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rows.wgsl"));

    Ok(())
}
