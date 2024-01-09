type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(err) = run() {
        eprintln!("error: {err}");
    }
}

fn run() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            el::KeyCode,
            glam::Vec4,
            sl::{self, Index, Out},
            update, Control, Frame,
        },
        std::f32::consts,
    };

    const COLOR: Vec4 = Vec4::new(1., 0., 0., 1.);
    const THIRD: f32 = consts::TAU / 3.;
    const R_OFFSET: f32 = -consts::TAU / 4.;
    const Y_OFFSET: f32 = 0.25;

    let triangle = |Index(index): Index| {
        let [x, y] = sl::thunk(sl::f32(index) * THIRD + R_OFFSET);
        Out {
            place: sl::vec4(sl::cos(x), sl::sin(y) + Y_OFFSET, 0., 1.),
            color: COLOR,
        }
    };

    let window = helpers::block_on(dunge::window().with_title("Triangle").make())?;
    let cx = window.context();
    let shader = cx.make_shader(triangle);
    let layer = cx.make_layer(window.format(), &shader);
    let update = |ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                ctrl.close();
            }
        }
    };

    let draw = |mut frame: Frame| {
        let clear = Rgba::from_standard([0., 0., 0., 1.]);
        frame.layer(&layer, clear).bind_empty().draw_triangles(1);
    };

    window.run(update::from_fn(update, draw))?;
    Ok(())
}
