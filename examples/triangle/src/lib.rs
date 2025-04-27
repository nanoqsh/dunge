type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::_window::WindowState) -> Result<(), Error> {
    use dunge::{
        color::Rgb,
        glam::Vec4,
        prelude::*,
        sl::{Groups, Index, Render},
        uniform::Uniform,
    };

    #[derive(Group)]
    struct Offset<'uni>(&'uni Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset<'_>>| {
        use std::f32::consts;

        let color = Vec4::new(1., 0.4, 0.8, 1.);
        let third = const { consts::TAU / 3. };

        let i = sl::thunk(sl::f32(idx) * third + offset.0);
        Render {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color,
        }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let mut r = 0.;
    let uniform = cx.make_uniform(&r);
    let set = cx.make_set(&shader, Offset(&uniform));

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
            uniform.update(&cx, &r);
            Then::Run
        };

        let draw = move |mut frame: _Frame<'_, '_>| {
            let opts = Rgb::from_standard([0.1, 0.05, 0.15]);
            frame._set_layer(&layer, opts)._bind(&set)._draw_points(3);
        };

        dunge::update(upd, draw)
    };

    ws.run(cx, dunge::make(make_handler))?;
    Ok(())
}
