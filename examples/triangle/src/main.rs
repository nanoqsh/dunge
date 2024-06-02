fn main() {
    env_logger::init();
    let ws = dunge::window().with_title("Triangle");
    if let Err(err) = helpers::block_on(triangle::run(ws)) {
        eprintln!("error: {err}");
    }
}
