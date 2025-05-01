fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::winit::try_block_on(cube::run) {
        eprintln!("error: {e}");
    }
}
