fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::try_block_on(renderbuffer::run) {
        eprintln!("error: {e}");
    }
}
