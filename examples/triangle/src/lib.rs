type Error = Box<dyn std::error::Error>;

pub fn run(window: dunge::window::Window) -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            glam::Vec4,
            prelude::*,
            sl::{Groups, Index, Out},
            uniform::Uniform,
        },
        std::f32::consts,
    };

    const COLOR: Vec4 = Vec4::new(1., 0.4, 0.8, 1.);
    const THIRD: f32 = consts::TAU / 3.;

    #[derive(Group)]
    struct Offset<'a>(&'a Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset>| {
        let [x, y] = sl::thunk(sl::f32(idx) * THIRD + offset.0);
        Out {
            place: sl::vec4(sl::cos(x), sl::sin(y), 0., 1.),
            color: COLOR,
        }
    };

    let cx = window.context();
    let shader = cx.make_shader(triangle);
    let mut r = 0.;
    let uniform = cx.make_uniform(r);
    let bind = {
        let offset = Offset(&uniform);
        let mut binder = cx.make_binder(&shader);
        binder.bind(&offset);
        binder.into_binding()
    };

    let layer = cx.make_layer(&shader, window.format());
    let upd = move |ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        r += ctrl.delta_time().as_secs_f32() * 0.5;
        uniform.update(&cx, r);
        Then::Run
    };

    let draw = move |mut frame: Frame| {
        let opts = Rgba::from_standard([0.1, 0.05, 0.15, 1.]);
        frame.layer(&layer, opts).bind(&bind).draw_points(3);
    };

    #[cfg(target_family = "wasm")]
    window.spawn(dunge::update(upd, draw));

    #[cfg(not(target_family = "wasm"))]
    window.run(dunge::update(upd, draw))?;

    Ok(())
}

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
pub async fn start() {
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let window = dunge::from_element("root").await;
    if let Err(err) = window.map_err(Box::from).and_then(run) {
        panic!("error: {err}");
    }
}
