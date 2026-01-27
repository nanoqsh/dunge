use {
    dunge_winit::prelude::*,
    std::{error, f32::consts, time::Duration},
};

type Error = Box<dyn error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge::{
            sl_old::{Groups, Index, Render},
            store::Uniform,
        },
        dunge_winit::Canvas,
        futures_concurrency::prelude::*,
        glam::Vec4,
        winit::keyboard::KeyCode,
    };

    let triangle = |Index(idx): Index, Groups(offset): Groups<Uniform<f32>>| {
        let color = Vec4::new(1., 0.4, 0.8, 1.);
        let third = const { consts::TAU / 3. };

        let i = sl_old::thunk(sl_old::f32(idx) * third + offset.load());
        Render {
            place: sl_old::vec4(sl_old::cos(i.clone()), sl_old::sin(i), 0., 1.),
            color,
        }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let offset = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, &offset);

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time| {
        time += delta_time;
        let t = time.as_secs_f32() * 0.5;
        offset.update(&cx, &t);
    };

    let window = control
        .make_window(&cx)
        .with_title("triangle")
        .with_canvas(Canvas::by_id("root"))
        .await?;

    let layer = cx.make_layer(&shader, window.format());

    let bg = window.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(redraw.delta_time());
            cx.shed(|s| {
                s.render(&redraw, bg).layer(&layer).set(&set).draw_points(3);
            })
            .await;

            redraw.present();
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);
    (render, close, esc_pressed).race().await;

    Ok(())
}
