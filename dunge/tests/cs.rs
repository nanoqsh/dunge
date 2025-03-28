#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn cs_empty() -> Result<(), Error> {
    use dunge::{
        sl::{self, Compute, Groups, Invocation},
        storage::Storage,
        Group,
    };

    #[derive(Group)]
    struct Map<'a> {
        array: &'a Storage<f32>,
    }

    let compute = |Invocation(_): Invocation, Groups(_): Groups<Map>| Compute {
        compute: sl::u32(0),
        workgroup_size: [64, 1, 1],
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("cs_empty.wgsl"));
    Ok(())
}
