#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| dunge::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn index() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 1]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.store(0u32, i.load(3u32).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("index.wgsl"));

    Ok(())
}

#[test]
fn index_u32() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::{RwStorage, Storage},
    };

    type Io = (Storage<[f32; 4]>, RwStorage<[f32; 1]>);

    let code = |Groups((i, o)): Groups<Io>| Compute {
        compute: o.store(0u32, i.load_with_u32(3).deref()),
        workgroup_size: [1; 3],
    };

    let shader = CONTEXT.make_shader(code);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("index_u32.wgsl"));

    Ok(())
}
