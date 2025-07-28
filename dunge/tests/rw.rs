#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn rw_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set(0u32, i.get(0u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_array.wgsl"));

    Ok(())
}

#[test]
fn rw_array_u32() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set_u32(0, i.get_u32(0).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_array_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_dyn_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32]>, RwStorage<[f32]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set(0u32, i.get(0u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_dyn_array.wgsl"));

    Ok(())
}

#[test]
fn rw_dyn_array_u32() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32]>, RwStorage<[f32]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set_u32(0, i.get_u32(0).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_dyn_array_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_vec() -> Result<(), Error> {
    use dunge::{
        glam::Vec3,
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<Vec3>, RwStorage<Vec3>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set(0u32, i.get(0u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_vec.wgsl"));

    Ok(())
}

#[test]
fn rw_vec_u32() -> Result<(), Error> {
    use dunge::{
        glam::Vec3,
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<Vec3>, RwStorage<Vec3>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set_u32(0, i.get_u32(0).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_vec_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_mat() -> Result<(), Error> {
    use dunge::{
        glam::Mat3,
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<Mat3>, RwStorage<Mat3>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set(0u32, i.get(0u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_mat.wgsl"));

    Ok(())
}

#[test]
fn rw_mat_u32() -> Result<(), Error> {
    use dunge::{
        glam::Mat3,
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<Mat3>, RwStorage<Mat3>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set_u32(0, i.get_u32(0).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_mat_u32.wgsl"));

    Ok(())
}
