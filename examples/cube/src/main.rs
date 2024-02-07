fn main() {
    env_logger::init();
    let window = helpers::block_on(dunge::window().with_title("Cube"));
    if let Err(err) = window.map_err(Box::from).and_then(cube::run) {
        eprintln!("error: {err}");
    }
}
