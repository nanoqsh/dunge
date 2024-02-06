fn main() {
    env_logger::init();
    if let Err(err) = helpers::block_on(triangle::run()) {
        eprintln!("error: {err}");
    }
}
