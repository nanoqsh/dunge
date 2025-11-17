#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn rw_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups, Ret},
        storage::{RwStorage, Storage},
        types,
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get(0u32);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set(0u32, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_array.wgsl"));

    Ok(())
}

#[test]
fn rw_array_u32() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups, Ret},
        storage::{RwStorage, Storage},
        types,
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get_u32(0);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set_u32(0, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_array_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_dyn_array() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups, Ret},
        storage::{RwStorage, Storage},
        types,
    };

    type Io = (Storage<[f32]>, RwStorage<[f32]>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get(0u32);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set(0u32, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_dyn_array.wgsl"));

    Ok(())
}

#[test]
fn rw_dyn_array_u32() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups, Ret},
        storage::{RwStorage, Storage},
        types,
    };

    type Io = (Storage<[f32]>, RwStorage<[f32]>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get_u32(0);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set_u32(0, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_dyn_array_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_vec() -> Result<(), Error> {
    use {
        dunge::{
            sl::{Compute, Groups, Ret},
            storage::{RwStorage, Storage},
            types,
        },
        glam::Vec3,
    };

    type Io = (Storage<Vec3>, RwStorage<Vec3>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get(0u32);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set(0u32, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_vec.wgsl"));

    Ok(())
}

#[test]
fn rw_vec_u32() -> Result<(), Error> {
    use {
        dunge::{
            sl::{Compute, Groups, Ret},
            storage::{RwStorage, Storage},
            types,
        },
        glam::Vec3,
    };

    type Io = (Storage<Vec3>, RwStorage<Vec3>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<f32>> = i.get_u32(0);
        let b: Ret<_, f32> = a.load();
        let c: Ret<_, f32> = o.set_u32(0, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_vec_u32.wgsl"));

    Ok(())
}

#[test]
fn rw_mat() -> Result<(), Error> {
    use {
        dunge::{
            sl::{Compute, Groups, Ret},
            storage::{RwStorage, Storage},
            types,
        },
        glam::Mat3,
    };

    type Io = (Storage<Mat3>, RwStorage<Mat3>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<types::Vec3<f32>>> = i.get(0u32);
        let b: Ret<_, types::Vec3<f32>> = a.load();
        let c: Ret<_, types::Vec3<f32>> = o.set(0u32, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_mat.wgsl"));

    Ok(())
}

#[test]
fn rw_mat_u32() -> Result<(), Error> {
    use {
        dunge::{
            sl::{Compute, Groups, Ret},
            storage::{RwStorage, Storage},
            types,
        },
        glam::Mat3,
    };

    type Io = (Storage<Mat3>, RwStorage<Mat3>);

    let code = |Groups((i, o)): Groups<Io>| {
        let a: Ret<_, types::Pointer<types::Vec3<f32>>> = i.get_u32(0);
        let b: Ret<_, types::Vec3<f32>> = a.load();
        let c: Ret<_, types::Vec3<f32>> = o.set_u32(0, b);

        Compute {
            compute: c,
            workgroup_size: [1; 3],
        }
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("rw_mat_u32.wgsl"));

    Ok(())
}
