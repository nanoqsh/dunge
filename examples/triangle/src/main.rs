type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(err) = helpers::block_on(run()) {
        eprintln!("error: {err}");
    }
}

async fn run() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            el::KeyCode,
            glam::Vec4,
            sl::{self, Groups, Index, Out},
            uniform::Uniform,
            update, Control, Frame, Group,
        },
        std::f32::consts,
    };

    const COLOR: Vec4 = Vec4::new(1., 0., 0., 1.);
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

    let window = dunge::window().with_title("Triangle").await?;
    let cx = window.context();
    let shader = cx.make_shader(triangle);
    let mut r = 0.;
    let uniform = cx.make_uniform(r);
    let bind = {
        let mut binder = cx.make_binder(&shader);
        let offset = Offset(&uniform);
        binder.bind(&offset);
        binder.into_binding()
    };

    let layer = cx.make_layer(window.format(), &shader);
    let update = |ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                ctrl.close();
            }
        }

        r += ctrl.delta_time().as_secs_f32();
        uniform.update(&cx, r);
    };

    let draw = |mut frame: Frame| {
        let clear = Rgba::from_standard([0., 0., 0., 1.]);
        frame.layer(&layer, clear).bind(&bind).draw_triangles(1);
    };

    window.run(update::from_fn(update, draw))?;
    Ok(())
}
