use dunge_winit::prelude::*;

type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::winit::try_block_on(run) {
        eprintln!("error: {e}");
    }
}

async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge_winit::{
            glam::{Vec2, Vec3},
            storage::Uniform,
        },
        futures_concurrency::prelude::*,
        futures_lite::prelude::*,
        std::{cell::Cell, time::Duration},
        winit::{keyboard::KeyCode, window},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec2,
        col: Vec3,
    }

    #[derive(Group)]
    struct Delta<'u>(&'u Uniform<f32>);

    let triangle = |vert: sl::InVertex<Vert>, sl::Groups(u): sl::Groups<Delta<'_>>| {
        let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));
        let fragment_col = sl::fragment(vert.col);
        let color = sl::vec4_with(fragment_col * u.0, 1.);
        sl::Render { place, color }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let uniform = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, Delta(&uniform));

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time: Duration| {
        time += delta_time;
        let v = time.as_secs_f32().sin() * 0.5 + 0.5;
        uniform.update(&cx, &v);
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

        let verts = MeshData::from_verts(&VERTS);
        cx.make_mesh(&verts)
    };

    let window = control.make_window(&cx, Attributes::default()).await?;
    let layer = cx.make_layer(&shader, window.format());

    let fps = Cell::new(0);
    let inc = || fps.set(fps.get() + 1);
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

            cx.shed(|mut s| {
                s.render(&redraw, bg).layer(&layer).set(&set).draw(&mesh);
            })
            .await;

            redraw.present();
            inc();
        }
    };

    let resize = async {
        loop {
            let (width, height) = window.resized().await;
            println!("resized: {width} {height}");
        }
    };

    let toggle_fullscreen = async {
        let mut fullscreen = false;
        loop {
            window.pressed(KeyCode::KeyF).await;

            fullscreen = !fullscreen;
            window.winit().set_fullscreen(if fullscreen {
                Some(window::Fullscreen::Borderless(None))
            } else {
                None
            });
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.pressed(KeyCode::Escape);

    (
        fps_counter,
        render,
        resize,
        toggle_fullscreen,
        close,
        esc_pressed,
    )
        .race()
        .await;

    Ok(())
}
