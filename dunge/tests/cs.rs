#![cfg(not(target_family = "wasm"))]

use {dunge::Context, std::sync::LazyLock};

static CONTEXT: LazyLock<Context> =
    LazyLock::new(|| helpers::block_on(dunge::context()).expect("failed to create dunge context"));

type Error = Box<dyn std::error::Error>;

#[test]
fn cs_array() -> Result<(), Error> {
    use dunge::{
        sl::{self, Compute, Groups},
        storage::Storage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<[f32; 4]>,
    }

    let compute = |Groups(_): Groups<Map>| Compute {
        // TODO: array operations
        compute: sl::u32(0),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array.wgsl"));
    Ok(())
}

#[test]
fn cs_array_mut() -> Result<(), Error> {
    use dunge::{
        sl::{self, Compute, Groups},
        storage::RwStorage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a RwStorage<[f32; 4]>,
    }

    let compute = |Groups(_): Groups<Map>| Compute {
        // TODO: array operations
        compute: sl::u32(0),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array_mut.wgsl"));
    Ok(())
}

#[test]
fn cs_dynamic_array() -> Result<(), Error> {
    use dunge::{
        sl::{self, Compute, Groups},
        storage::Storage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<[f32]>,
    }

    let compute = |Groups(_): Groups<Map>| Compute {
        // TODO: array operations
        compute: sl::u32(0),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_dynamic_array.wgsl"));
    Ok(())
}

#[test]
fn cs_dynamic_array_mut() -> Result<(), Error> {
    use dunge::{
        sl::{self, Compute, Groups},
        storage::RwStorage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a RwStorage<[f32]>,
    }

    let compute = |Groups(_): Groups<Map>| Compute {
        // TODO: array operations
        compute: sl::u32(0),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(
        shader.debug_wgsl(),
        include_str!("cs_dynamic_array_mut.wgsl"),
    );

    Ok(())
}

#[test]
fn cs_array2d() -> Result<(), Error> {
    use dunge::{
        sl::{Compute, Groups},
        storage::Storage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<[[f32; 4]; 4]>,
    }

    let compute = |Groups(m): Groups<Map>| Compute {
        compute: m.array.load(0u32).load(0u32),
        workgroup_size: [64, 1, 1],
    };

    let shader = CONTEXT.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_array2d.wgsl"));
    Ok(())
}
