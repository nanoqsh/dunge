use dunge::{winit::platform::android::activity::AndroidApp, CanvasConfig};

#[no_mangle]
fn android_main(app: AndroidApp) {
    use {android_logger::Config, log::LevelFilter, scene::App};

    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    log::info!("start");

    dunge::from_app(app)
        .run_blocking(CanvasConfig::default(), App::new)
        .expect("loop error");
}
