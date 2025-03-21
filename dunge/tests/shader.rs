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
        let mt = sl::thunk(m);
        let v = mt.clone().x() + mt.clone().y();
        let z = sl::splat_vec3(1.).z();

        Out {
            place: sl::vec4_concat(mt.x(), v) * sl::f32(1) * z,
            color: sl::vec4(0., 0., 1., 1.) + Vec4::splat(0.),
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_calc.wgsl"));
    Ok(())
}

#[test]
fn shader_if() -> Result<(), Error> {
    use dunge::{
        glam::Vec4,
        sl::{self, Out},
    };

    let compute = || Out {
        place: sl::if_then_else(true, || Vec4::splat(3.), || sl::splat_vec4(2.) * 2.),
        color: sl::splat_vec4(1.),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_if.wgsl"));
    Ok(())
}

#[test]
fn shader_branch() -> Result<(), Error> {
    use dunge::sl::{self, Out};

    let cx = helpers::block_on(dunge::context())?;
    let shader0 = {
        let compute = || Out {
            place: sl::default(|| sl::splat_vec4(1.)).when(false, || sl::splat_vec4(2.)),
            color: sl::splat_vec4(1.),
        };

        cx.make_shader(compute)
    };

    let shader1 = {
        let compute = || Out {
            place: sl::default(|| sl::splat_vec4(1.))
                .when(true, || sl::splat_vec4(2.))
                .when(false, || sl::splat_vec4(3.)),
            color: sl::splat_vec4(1.),
        };

        cx.make_shader(compute)
    };

    let shader2 = {
        let compute = || {
            let p = sl::default(|| sl::splat_vec4(1.))
                .when(true, || sl::splat_vec4(2.))
                .when(true, || sl::splat_vec4(3.))
                .when(false, || sl::splat_vec4(4.));

            Out {
                place: p,
                color: sl::splat_vec4(1.),
            }
        };

        cx.make_shader(compute)
    };

    helpers::eq_lines(shader0.debug_wgsl(), include_str!("shader_branch0.wgsl"));
    helpers::eq_lines(shader1.debug_wgsl(), include_str!("shader_branch1.wgsl"));
    helpers::eq_lines(shader2.debug_wgsl(), include_str!("shader_branch2.wgsl"));
    Ok(())
}

#[test]
fn shader_discard() -> Result<(), Error> {
    use dunge::sl::{self, Out};

    let cx = helpers::block_on(dunge::context())?;
    let compute = || Out {
        place: sl::splat_vec4(1.),
        color: sl::discard(),
    };

    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_discard.wgsl"));
    Ok(())
}

#[test]
fn shader_discard_if() -> Result<(), Error> {
    use dunge::sl::{self, Out};

    let cx = helpers::block_on(dunge::context())?;
    let compute = || Out {
        place: sl::splat_vec4(1.),
        color: sl::if_then_else(true, sl::discard, || sl::splat_vec4(1.)),
    };

    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_discard_if.wgsl"));
    Ok(())
}

#[test]
fn shader_zero() -> Result<(), Error> {
    use dunge::sl::{self, Out};

    let cx = helpers::block_on(dunge::context())?;
    let compute = || Out {
        place: sl::zero_value(),
        color: sl::zero_value(),
    };

    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_zero.wgsl"));
    Ok(())
}

#[test]
#[should_panic(expected = "thunk cannot be created outside of a shader function")]
fn shader_thunk_outside() {
    use dunge::sl::{self, Eval, Vs};

    fn make() -> impl Eval<Vs> {
        sl::thunk(1.)
    }

    _ = make();
}

#[test]
#[should_panic(expected = "reentrant in a shader function isn't allowed")]
fn shader_reentrant() {
    use dunge::sl::{self, Out};

    let cx = helpers::block_on(dunge::context()).expect("create context");
    let compute = {
        let cx = cx.clone();
        let inner = || Out {
            place: sl::splat_vec4(1.),
            color: sl::splat_vec4(1.),
        };

        move || {
            _ = cx.make_shader(inner);
            Out {
                place: sl::splat_vec4(1.),
                color: sl::splat_vec4(1.),
            }
        }
    };

    _ = cx.make_shader(compute);
}

#[test]
fn shader_storage() -> Result<(), Error> {
    use dunge::sl::{self, Groups, Index, Out};
    use dunge::{storage::Storage, Group};

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<f32>,
    }

    let compute = |Groups(map): Groups<Map>, Index(index): Index| Out {
        place: sl::splat_vec4(1.) * map.array.index(index),
        color: sl::splat_vec4(1.),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("shader_storage.wgsl"));
    Ok(())
}
