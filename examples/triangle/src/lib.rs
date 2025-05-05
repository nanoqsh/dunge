use dunge_winit::prelude::*;

type Error = Box<dyn std::error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge_winit::{
            glam::Vec4,
            sl::{Groups, Index, Render},
            storage::Uniform,
            winit::Canvas,
        },
        futures_concurrency::prelude::*,
        std::{f32::consts, time::Duration},
        winit::keyboard::KeyCode,
    };

    #[derive(Group)]
    struct Offset<'uni>(&'uni Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset<'_>>| {
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
    let uniform = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, Offset(&uniform));

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time| {
        time += delta_time;
        let t = time.as_secs_f32() * 0.5;
        uniform.update(&cx, &t);
    };

    let window = {
        let attr = Attributes::default()
            .with_title("triangle")
            .with_canvas(Canvas::by_id("root"));

        control.make_window(&cx, attr).await?
    };

    let layer = cx.make_layer(&shader, window.format());

    let bg = window.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(redraw.delta_time());
            cx.shed(|mut s| {
                s.render(&redraw, bg).layer(&layer).set(&set).draw_points(3);
            })
            .await;

            redraw.present();
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.pressed(KeyCode::Escape);
    (render, close, esc_pressed).race().await;

    Ok(())
}
