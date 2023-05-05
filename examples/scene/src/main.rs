fn main() {
    use {
        dunge::{Backend, BackendSelector, CanvasConfig, InitialState, WindowMode},
        scene::App,
    };

    env_logger::init();

    let config = CanvasConfig {
        backend_selector: BackendSelector::select_backend(Backend::Vulkan),
    };

    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 500,
            height: 500,
        },
        show_cursor: false,
        ..Default::default()
    })
    .run_blocking(config, App::new)
    .log_error();
}
