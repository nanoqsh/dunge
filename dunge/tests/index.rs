#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        Context,
        storage::{RwStorage, Storage},
    },
    std::sync::LazyLock,
};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);

#[test]
fn index() -> Result<(), Error> {
    use dunge::sl::{Compute, Groups};

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set(0u32, i.get(0u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("index.wgsl"));

    Ok(())
}

#[test]
fn index_u32() -> Result<(), Error> {
    use dunge::sl::{Compute, Groups};

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.set_with_u32(0, i.get_with_u32(0).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("index_u32.wgsl"));

    Ok(())
}
