#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn rs_calc() -> Result<(), Error> {
    use dunge::{
        glam::Vec4,
        sl::{self, Render},
    };

    let compute = || {
        let m = -sl::mat2(sl::vec2(1., 0.), sl::vec2(0., 1.));
        let mt = sl::thunk(m);
        let v = mt.clone().x() + mt.clone().y();
        let z = sl::splat_vec3(1.).z();

        Render {
            place: sl::vec4_concat(mt.x(), v) * sl::f32(1) * z,
            color: sl::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
        }
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_calc.wgsl"));
    Ok(())
}

#[test]
fn rs_if() -> Result<(), Error> {
    use dunge::{
        glam::Vec4,
        sl::{self, Render},
    };

    let compute = || Render {
        place: sl::if_then_else(true, || Vec4::splat(3.), || sl::splat_vec4(2.) * 2.),
        color: sl::splat_vec4(1.),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_if.wgsl"));
    Ok(())
}

#[test]
fn rs_branch() -> Result<(), Error> {
    use dunge::sl::{self, Render};

    let shader0 = {
        let compute = || Render {
            place: sl::default(|| sl::splat_vec4(1.)).when(false, || sl::splat_vec4(2.)),
            color: sl::splat_vec4(1.),
        };

        CONTEXT.make_shader(compute)
    };

    let shader1 = {
        let compute = || Render {
            place: sl::default(|| sl::splat_vec4(1.))
                .when(true, || sl::splat_vec4(2.))
                .when(false, || sl::splat_vec4(3.)),
            color: sl::splat_vec4(1.),
        };

        CONTEXT.make_shader(compute)
    };

    let shader2 = {
        let compute = || {
            let p = sl::default(|| sl::splat_vec4(1.))
                .when(true, || sl::splat_vec4(2.))
                .when(true, || sl::splat_vec4(3.))
                .when(false, || sl::splat_vec4(4.));

            Render {
                place: p,
                color: sl::splat_vec4(1.),
            }
        };

        CONTEXT.make_shader(compute)
    };

    helpers::eq_lines(shader0.debug_wgsl(), include_str!("rs_branch0.wgsl"));
    helpers::eq_lines(shader1.debug_wgsl(), include_str!("rs_branch1.wgsl"));
    helpers::eq_lines(shader2.debug_wgsl(), include_str!("rs_branch2.wgsl"));
    Ok(())
}

#[test]
fn rs_discard() -> Result<(), Error> {
    use dunge::sl::{self, Render};

    let compute = || Render {
        place: sl::splat_vec4(1.),
        color: sl::discard(),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_discard.wgsl"));
    Ok(())
}

#[test]
fn rs_discard_if() -> Result<(), Error> {
    use dunge::sl::{self, Render};

    let compute = || Render {
        place: sl::splat_vec4(1.),
        color: sl::if_then_else(true, sl::discard, || sl::splat_vec4(1.)),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_discard_if.wgsl"));
    Ok(())
}

#[test]
fn rs_zero() -> Result<(), Error> {
    use dunge::sl::{self, Render};

    let compute = || Render {
        place: sl::zero_value(),
        color: sl::zero_value(),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_zero.wgsl"));
    Ok(())
}

#[test]
#[should_panic(expected = "thunk cannot be created outside of a shader function")]
fn rs_thunk_outside() {
    use dunge::sl::{self, Eval, Vs};

    fn make() -> impl Eval<Vs> {
        sl::thunk(1.)
    }

    _ = make();
}

#[test]
#[should_panic(expected = "reentrant in a shader function isn't allowed")]
fn rs_reentrant() {
    use dunge::sl::{self, Render};

    let compute = {
        let cx = CONTEXT.clone();
        let inner = || Render {
            place: sl::splat_vec4(1.),
            color: sl::splat_vec4(1.),
        };

        move || {
            _ = cx.make_shader(inner);
            Render {
                place: sl::splat_vec4(1.),
                color: sl::splat_vec4(1.),
            }
        }
    };

    _ = CONTEXT.make_shader(compute);
}

#[test]
fn rs_storage() -> Result<(), Error> {
    use dunge::{
        Group,
        sl::{self, Groups, Index, Render},
        storage::Storage,
    };

    #[derive(Group)]
    struct Map {
        array: Storage<[f32; 4]>,
    }

    let compute = |Groups(map): Groups<Map>, Index(index): Index| Render {
        place: sl::splat_vec4(1.) * map.array.load(index),
        color: sl::splat_vec4(1.),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_storage.wgsl"));
    Ok(())
}

#[test]
fn rs_dyn() -> Result<(), Error> {
    use dunge::sl::{self, Render};

    for (do_sin, correct_shader) in [
        (true, include_str!("rs_dyn_true.wgsl")),
        (false, include_str!("rs_dyn_false.wgsl")),
    ] {
        let compute = |sl::Index(index): sl::Index| {
            let new_val = if do_sin {
                sl::thunk(sl::sin(sl::f32(index)))
            } else {
                sl::thunk(sl::f32(index))
            };

            Render {
                place: sl::splat_vec4(new_val),
                color: sl::splat_vec4(1.),
            }
        };

        let shader = CONTEXT.make_shader(compute);
        helpers::eq_lines(shader.debug_wgsl(), correct_shader);
    }
    Ok(())
}
