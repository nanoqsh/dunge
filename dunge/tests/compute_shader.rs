#![cfg(not(target_family = "wasm"))]

use dunge::{
    sl::{GlobalInvocationId, Groups},
    storage::Storage,
    types::StorageReadWrite,
    Group,
};

type Error = Box<dyn std::error::Error>;

#[test]
fn simple_compute_shader() -> Result<(), Error> {
    use dunge::sl::{self, CsOut};

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<f32, StorageReadWrite>,
    }

    let compute = |GlobalInvocationId(inv): GlobalInvocationId, Groups(map): Groups<Map>| {
        let set = map.array.set_index(inv.x(), sl::f32(1.0f32));
        CsOut { compute: set }
    };

    let cx = helpers::block_on(dunge::context())?;
    let module = cx.make_compute_shader(compute);
    helpers::eq_lines(&module.wgsl, include_str!("simple_compute_shader.wgsl"));
    Ok(())
}
