fn main() {
    env_logger::init();
    let ws = dunge::window_state().with_title("Cube");
    if let Err(err) = helpers::block_on(cube::run(ws)) {
        eprintln!("error: {err}");
    }
}
