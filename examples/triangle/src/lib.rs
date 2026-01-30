pub mod shader;

use {
    dunge::store::Uniform,
    dunge_winit::{Canvas, prelude::*},
    futures_concurrency::prelude::*,
    std::{error, time::Duration},
    winit::keyboard::KeyCode,
};

type Error = Box<dyn error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    let cx = dunge::context().await?;
    let triangle = cx.make_shader(render! {
        groups: [Uniform<f32>],
        shaders: [shader::vs, shader::fs],
    }?);

    let offset = cx.make_uniform(&0.);
    let set = cx.make_set(&triangle, &offset);

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

    let layer = cx.make_layer(&triangle, window.format());

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
