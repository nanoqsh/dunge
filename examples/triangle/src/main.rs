fn main() {
    env_logger::init();
    let window = helpers::block_on(dunge::window().with_title("Triangle"));
    if let Err(err) = window.map_err(Box::from).and_then(triangle::run) {
        eprintln!("error: {err}");
    }
}
