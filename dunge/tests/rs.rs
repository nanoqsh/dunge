#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn rs_calc() -> Result<(), Error> {
    use {
        dunge::sl_old::{self, Render},
        glam::Vec4,
    };

    let compute = || {
        let m = -sl_old::mat2(sl_old::vec2(1., 0.), sl_old::vec2(0., 1.));
        let mt = sl_old::thunk(m);
        let v = mt.clone().x() + mt.clone().y();
        let z = sl_old::vec3_splat(1.).z();

        Render {
            place: sl_old::vec4_concat(mt.x(), v) * sl_old::f32(1) * z,
            color: sl_old::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
        }
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_calc.wgsl"));
    Ok(())
}

#[test]
fn rs_if() -> Result<(), Error> {
    use {
        dunge::sl_old::{self, Render},
        glam::Vec4,
    };

    let compute = || Render {
        place: sl_old::if_then_else(true, || Vec4::splat(3.), || sl_old::vec4_splat(2.) * 2.),
        color: sl_old::vec4_splat(1.),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_if.wgsl"));
    Ok(())
}

#[test]
fn rs_branch() -> Result<(), Error> {
    use dunge::sl_old::{self, Render};

    let shader0 = {
        let compute = || Render {
            place: sl_old::default(|| sl_old::vec4_splat(1.))
                .when(false, || sl_old::vec4_splat(2.)),
            color: sl_old::vec4_splat(1.),
        };

        CONTEXT.make_shader(compute)
    };

    let shader1 = {
        let compute = || Render {
            place: sl_old::default(|| sl_old::vec4_splat(1.))
                .when(true, || sl_old::vec4_splat(2.))
                .when(false, || sl_old::vec4_splat(3.)),
            color: sl_old::vec4_splat(1.),
        };

        CONTEXT.make_shader(compute)
    };

    let shader2 = {
        let compute = || {
            let p = sl_old::default(|| sl_old::vec4_splat(1.))
                .when(true, || sl_old::vec4_splat(2.))
                .when(true, || sl_old::vec4_splat(3.))
                .when(false, || sl_old::vec4_splat(4.));

            Render {
                place: p,
                color: sl_old::vec4_splat(1.),
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
    use dunge::sl_old::{self, Render};

    let compute = || Render {
        place: sl_old::vec4_splat(1.),
        color: sl_old::discard(),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_discard.wgsl"));
    Ok(())
}

#[test]
fn rs_discard_if() -> Result<(), Error> {
    use dunge::sl_old::{self, Render};

    let compute = || Render {
        place: sl_old::vec4_splat(1.),
        color: sl_old::if_then_else(true, sl_old::discard, || sl_old::vec4_splat(1.)),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_discard_if.wgsl"));
    Ok(())
}

#[test]
fn rs_zero() -> Result<(), Error> {
    use dunge::sl_old::{self, Render};

    let compute = || Render {
        place: sl_old::zero_value(),
        color: sl_old::zero_value(),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_zero.wgsl"));
    Ok(())
}

#[test]
#[should_panic(expected = "thunk cannot be created outside of a shader function")]
fn rs_thunk_outside() {
    use dunge::sl_old::{self, Eval, Vs};

    fn make() -> impl Eval<Vs> {
        sl_old::thunk(1.)
    }

    _ = make();
}

#[test]
#[should_panic(expected = "reentrant in a shader function isn't allowed")]
fn rs_reentrant() {
    use dunge::sl_old::{self, Render};

    let compute = {
        let cx = CONTEXT.clone();
        let inner = || Render {
            place: sl_old::vec4_splat(1.),
            color: sl_old::vec4_splat(1.),
        };

        move || {
            _ = cx.make_shader(inner);
            Render {
                place: sl_old::vec4_splat(1.),
                color: sl_old::vec4_splat(1.),
            }
        }
    };

    _ = CONTEXT.make_shader(compute);
}

#[test]
fn rs_storage() -> Result<(), Error> {
    use dunge::{
        GroupLegacy,
        sl_old::{self, Groups, Index, Render},
        store::StorageOld,
    };

    #[derive(GroupLegacy)]
    struct Map {
        array: StorageOld<[f32; 4]>,
    }

    let compute = |Groups(map): Groups<Map>, Index(index): Index| Render {
        place: sl_old::vec4_splat(1.) * map.array.get(index).load(),
        color: sl_old::vec4_splat(1.),
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rs_storage.wgsl"));
    Ok(())
}

#[test]
fn rs_dyn() -> Result<(), Error> {
    use dunge::sl_old::{self, Render};

    for (do_sin, correct_shader) in [
        (true, include_str!("rs_dyn_true.wgsl")),
        (false, include_str!("rs_dyn_false.wgsl")),
    ] {
        let compute = |sl_old::Index(index): sl_old::Index| {
            let new_val = if do_sin {
                sl_old::thunk(sl_old::sin(sl_old::f32(index)))
            } else {
                sl_old::thunk(sl_old::f32(index))
            };

            Render {
                place: sl_old::vec4_splat(new_val),
                color: sl_old::vec4_splat(1.),
            }
        };

        let shader = CONTEXT.make_shader(compute);
        helpers::eq_lines(shader.debug_wgsl(), correct_shader);
    }
    Ok(())
}
