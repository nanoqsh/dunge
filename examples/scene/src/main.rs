fn main() -> ! {
    use {
        dunge::{CanvasConfig, InitialState, WindowMode},
        scene::App,
    };

    env_logger::init();
    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 500,
            height: 500,
        },
        show_cursor: false,
        ..Default::default()
    })
    .run_blocking(CanvasConfig::default(), App::new)
    .into_panic()
}
