use {
    dunge::{
        context::{Context, Error},
        sl::{Index, Out},
    },
    futures::future,
    glam::Vec4,
};

#[test]
fn render() -> Result<(), Error> {
    let triangle = |Index(_): Index| Out {
        place: Vec4::splat(1.),
        color: Vec4::splat(1.),
    };

    let cx = future::block_on(Context::new())?;
    _ = cx.make_shader(triangle);
    Ok(())
}
