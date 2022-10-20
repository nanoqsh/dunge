fn main() {
    use {
        cube::App,
        dunge::{InitialState, WindowMode},
    };

    env_logger::init();
    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 500,
            height: 500,
        },
        ..Default::default()
    })
    .run_blocking(App::new);
}
