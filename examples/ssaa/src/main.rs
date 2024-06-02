fn main() {
    env_logger::init();
    let ws = dunge::window_state().with_title("SSAA");
    if let Err(err) = helpers::block_on(ssaa::run(ws)) {
        eprintln!("error: {err}");
    }
}
