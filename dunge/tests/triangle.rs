#[test]
fn render() {
    use {dunge::context::Context, futures::future};

    let cx = future::block_on(Context::new()).expect("create context");
    _ = cx;
}
