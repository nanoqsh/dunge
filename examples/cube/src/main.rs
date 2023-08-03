fn main() -> ! {
    use {
        cube::App,
        dunge::{CanvasConfig, InitialState, WindowMode},
    };

    env_logger::init();
    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 500,
            height: 500,
        },
        ..Default::default()
    })
    .run_blocking(CanvasConfig::default(), App::new)
    .into_panic()
}
