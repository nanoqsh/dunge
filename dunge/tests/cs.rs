#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn cs_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::Storage,
    };

    let compute = |Groups(a): Groups<Storage<[f32; 4]>>| Compute {
        compute: a.load(0u32),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array.wgsl"));
    Ok(())
}

#[test]
fn cs_array_rw() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::RwStorage,
    };

    let compute = |Groups(a): Groups<RwStorage<[f32; 4]>>| Compute {
        compute: a.store(0u32, 1.),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array_rw.wgsl"));
    Ok(())
}

#[test]
fn cs_dynamic_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::Storage,
    };

    let compute = |Groups(a): Groups<Storage<[f32]>>| Compute {
        compute: a.load(0u32),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_dynamic_array.wgsl"));
    Ok(())
}

#[test]
fn cs_dynamic_array_rw() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::RwStorage,
    };

    let compute = |Groups(a): Groups<RwStorage<[f32]>>| Compute {
        compute: a.store(0u32, 1.),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(
        shader.debug_wgsl(),
        include_str!("cs_dynamic_array_rw.wgsl"),
    );

    Ok(())
}

#[test]
fn cs_array2d() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::Storage,
    };

    let compute = |Groups(a): Groups<Storage<[[f32; 4]; 4]>>| Compute {
        compute: a.load(0u32).load(0u32),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array2d.wgsl"));
    Ok(())
}
