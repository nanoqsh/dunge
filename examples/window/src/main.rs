use dunge_winit::prelude::*;

type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::try_block_on(run) {
        eprintln!("error: {e}");
    }
}

async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge::{
            sl::{Groups, PassVertex, Render},
            storage::Uniform,
        },
        futures_concurrency::prelude::*,
        futures_lite::prelude::*,
        glam::{Vec2, Vec3},
        std::{cell::Cell, time::Duration},
        winit::{event::MouseButton, keyboard::KeyCode, window},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec2,
        col: Vec3,
    }

    let triangle = |PassVertex(v): PassVertex<Vert>, Groups(u): Groups<Uniform<f32>>| {
        let place = sl::vec4_concat(v.pos, sl::vec2(0., 1.));
        let fragment_col = sl::fragment(v.col);
        let color = sl::vec4_append(fragment_col * u.load(), 1.);
        Render { place, color }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let delta = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, &delta);

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time: Duration| {
        time += delta_time;
        let v = time.as_secs_f32().sin() * 0.5 + 0.5;
        delta.update(&cx, &v);
    };

    let mesh = {
        const VERTS: [Vert; 3] = [
            Vert {
                pos: Vec2::new(-0.5, -0.5),
                col: Vec3::new(1., 0., 0.),
            },
            Vert {
                pos: Vec2::new(0.5, -0.5),
                col: Vec3::new(0., 1., 0.),
            },
            Vert {
                pos: Vec2::new(0., 0.5),
                col: Vec3::new(0., 0., 1.),
            },
        ];

        cx.make_mesh(&MeshData::from_verts(&VERTS).expect("mesh data"))
    };

    let window = control.make_window(&cx).await?;
    let layer = cx.make_layer(&shader, window.format());

    let fps = Cell::new(0);
    let inc = || fps.update(|n| n + 1);
    let reset = || fps.take();

    let fps_counter = Duration::from_secs(1).interval().for_each(|_| {
        let total = reset();
        println!("fps: {total}");
    });

    let bg = layer.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(redraw.delta_time());

            cx.shed(|s| {
                s.render(&redraw, bg).layer(&layer).set(&set).draw(&mesh);
            })
            .await;

            redraw.present();
            inc();
        }
    };

    let resize = async {
        loop {
            let new_size = window.resized().await;
            println!("resized: {new_size}");
        }
    };

    let mut click_counter = 0;
    let click = async {
        loop {
            window.button_pressed(MouseButton::Left).await;
            click_counter += 1;
            println!("clicked {click_counter} times");
        }
    };

    let click_more = async {
        loop {
            (
                window.button_pressed(MouseButton::Left),
                window.button_pressed(MouseButton::Right),
            )
                .race()
                .await;

            println!("clicked");
        }
    };

    let toggle_fullscreen = async {
        let mut fullscreen = false;
        loop {
            window.key_pressed(KeyCode::KeyF).await;

            fullscreen = !fullscreen;
            window.winit().set_fullscreen(if fullscreen {
                Some(window::Fullscreen::Borderless(None))
            } else {
                None
            });
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);

    (
        async {
            match (
                fps_counter,
                render,
                resize,
                click,
                click_more,
                toggle_fullscreen,
            )
                .join()
                .await {}
        },
        close,
        esc_pressed,
    )
        .race()
        .await;

    Ok(())
}
