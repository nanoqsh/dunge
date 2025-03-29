type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::window::WindowState) -> Result<(), Error> {
    use dunge::{
        color::Rgba,
        glam::Vec4,
        prelude::*,
        sl::{Groups, Index, Out},
        uniform::Uniform,
    };

    #[derive(Group)]
    struct Offset<'a>(&'a Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset>| {
        use std::f32::consts;

        let color = const { Vec4::new(1., 0.4, 0.8, 1.) };
        let third = const { consts::TAU / 3. };

        let i = sl::thunk(sl::f32(idx) * third + offset.0);
        Out {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color,
        }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_render_shader(triangle);
    let mut r = 0.;
    let uniform = cx.make_uniform(r);
    let bind = {
        let offset = Offset(&uniform);
        let mut binder = cx.make_binder(&shader);
        binder.add(&offset);
        binder.into_binding()
    };

    let make_handler = move |cx: &Context, view: &View| {
        let layer = cx.make_layer(&shader, view.format());

        let cx = cx.clone();
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

        dunge::update(upd, draw)
    };

    ws.run(cx, dunge::make(make_handler))?;
    Ok(())
}
