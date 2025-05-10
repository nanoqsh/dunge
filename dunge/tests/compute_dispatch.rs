#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn compute() -> Result<(), Error> {
    use {
        dunge::{
            Group,
            buffer::BufferData,
            sl::{Compute, Groups, Invocation},
            storage::RwStorage,
        },
        std::iter,
    };

    const SIZE: u32 = 16;
    const STORAGE_SIZE: usize = SIZE as usize * SIZE as usize;

    #[derive(Group)]
    struct Map<'store> {
        array: &'store RwStorage<[u32; STORAGE_SIZE]>,
    }

    let compute = |Invocation(v): Invocation, Groups(m): Groups<Map<'_>>| Compute {
        compute: m.array.store(v.x(), v.x()),
        workgroup_size: [SIZE, 1, 1],
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(compute);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("compute_dispatch.wgsl"));

    let storage = cx.make_storage(&[0; STORAGE_SIZE]).rw();
    let map = {
        let map = Map { array: &storage };
        cx.make_set(&shader, map)
    };

    let workload = cx.make_workload(&shader);

    // buffer to download compute result
    let mut download = cx.make_buffer(
        BufferData::empty((STORAGE_SIZE * size_of::<u32>()) as u32)
            .copy_to()
            .read(),
    );

    let read = dunge::block_on(async {
        cx.shed(|mut s| {
            s.compute()
                .workload(&workload)
                .set(&map)
                .dispatch(SIZE, 1, 1);

            s.copy(&storage, &download);
        })
        .await;

        cx.read(&mut download).await
    })?;

    let data: &[u32] = bytemuck::cast_slice(&read);
    for (i, &n) in iter::zip(0.., data) {
        assert_eq!(i, n);
    }

    Ok(())
}
