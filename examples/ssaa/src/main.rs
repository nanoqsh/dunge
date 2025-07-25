fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::try_block_on(ssaa::run) {
        eprintln!("error: {e}");
    }
}
