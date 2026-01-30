use {
    dunge::store::Uniform,
    dunge_winit::prelude::*,
    futures_concurrency::prelude::*,
    futures_lite::prelude::*,
    glam::{Vec2, Vec3, Vec4},
    std::{cell::Cell, error, time::Duration},
    winit::{event::MouseButton, keyboard::KeyCode, window},
};

fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::try_block_on(run) {
        eprintln!("error: {e}");
    }
}

#[derive(Clone, Copy, Value, Bytes)]
struct Vert {
    pos: Vec2,
    col: Vec3,
}

#[derive(Clone, Copy, Value)]
struct Io {
    #[position]
    pos: Vec4,
    col: Vec3,
}

#[dunge(vertex)]
fn vs(v: Vert) -> Io {
    Io {
        pos: sl::concat(v.pos, Vec2::new(0., 1.)),
        col: v.col,
    }
}

#[dunge(fragment)]
fn fs(io: Io, u: Uniform<f32>) -> Vec4 {
    sl::append(io.col * u.read(), 1.)
}

type Error = Box<dyn error::Error>;

async fn run(control: Control) -> Result<(), Error> {
    let cx = dunge::context().await?;
    let triangle = cx.make_shader(render! {
        vertex: Vert,
        groups: [Uniform<f32>],
        shaders: [vs, fs],
    }?);

    let delta = cx.make_uniform(&0.);
    let set = cx.make_set(&triangle, &delta);

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

        let data = MeshData::from_verts(&VERTS).expect("mesh data");
        cx.make_mesh(&data)
    };

    let window = control.make_window(&cx).await?;
    let layer = cx.make_layer(&triangle, window.format());

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

    let input = window.text_input().for_each(|s| println!("input: {s}"));

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
                input,
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
