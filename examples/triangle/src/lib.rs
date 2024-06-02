type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::window::WindowState) -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            glam::Vec4,
            prelude::*,
            sl::{Groups, Index, Out},
            uniform::Uniform,
        },
        std::{cell::OnceCell, f32::consts},
    };

    const COLOR: Vec4 = Vec4::new(1., 0.4, 0.8, 1.);
    const THIRD: f32 = consts::TAU / 3.;

    #[derive(Group)]
    struct Offset<'a>(&'a Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset>| {
        let i = sl::thunk(sl::f32(idx) * THIRD + offset.0);
        Out {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color: COLOR,
        }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let mut r = 0.;
    let uniform = cx.make_uniform(r);
    let bind = {
        let offset = Offset(&uniform);
        let mut binder = cx.make_binder(&shader);
        binder.bind(&offset);
        binder.into_binding()
    };

    let upd = {
        let cx = cx.clone();
        move |ctrl: &Control| {
            for key in ctrl.pressed_keys() {
                if key.code == KeyCode::Escape {
                    return Then::Close;
                }
            }

            r += ctrl.delta_time().as_secs_f32() * 0.5;
            uniform.update(&cx, r);
            Then::Run
        }
    };

    let draw = {
        let cx = cx.clone();
        let layer = OnceCell::default();
        move |mut frame: Frame| {
            let opts = Rgba::from_standard([0.1, 0.05, 0.15, 1.]);
            let layer = layer.get_or_init(|| cx.make_layer(&shader, frame.format()));
            frame.layer(layer, opts).bind(&bind).draw_points(3);
        }
    };

    ws.run(cx, dunge::update(upd, draw))?;
    Ok(())
}
