fn main() {
    env_logger::init();
    let ws = dunge::window().with_title("Triangle");
    if let Err(e) = dunge::block_on(triangle::run(ws)) {
        eprintln!("error: {e}");
    }
}
