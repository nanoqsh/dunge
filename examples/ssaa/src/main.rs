fn main() {
    env_logger::init();
    let ws = dunge::window().with_title("SSAA");
    if let Err(err) = dunge::block_on(ssaa::run(ws)) {
        eprintln!("error: {err}");
    }
}
