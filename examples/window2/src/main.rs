use dunge_winit::{runtime::Control, winit};

type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::runtime::try_block_on(run) {
        eprintln!("error: {e}");
    }
}

async fn run(ctrl: Control<'_>) -> Result<(), Error> {
    use {
        dunge_winit::{
            color::Rgb, prelude::*, runtime::Attributes, uniform::Uniform, winit::keyboard::KeyCode,
        },
        futures_concurrency::prelude::*,
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: [f32; 2],
        col: [f32; 3],
    }

    #[derive(Group)]
    struct Delta<'u>(&'u Uniform<f32>);

    let triangle = |vert: sl::InVertex<Vert>, sl::Groups(u): sl::Groups<Delta<'_>>| {
        let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));
        let fragment_col = sl::fragment(vert.col);
        let color = sl::vec4_with(fragment_col * u.0, 1.);
        sl::Render { place, color }
    };

    let cx = ctrl.context();
    let shader = cx.make_shader(triangle);

    let uniform = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, Delta(&uniform));

    let mesh = {
        let verts = const {
            MeshData::from_verts(&[
                Vert {
                    pos: [-0.5, -0.5],
                    col: [1., 0., 0.],
                },
                Vert {
                    pos: [0.5, -0.5],
                    col: [0., 1., 0.],
                },
                Vert {
                    pos: [0., 0.5],
                    col: [0., 0., 1.],
                },
            ])
        };

        cx.make_mesh(&verts)
    };

    let window = ctrl.make_window(Attributes::default()).await?;
    let layer = cx.make_layer(&shader, window.format());

    let render = async {
        let mut t = 0.;
        loop {
            let redraw = window.redraw().await;
            t += redraw.delta_time().as_secs_f32();
            let v = f32::sin(t) * 0.5 + 0.5;
            uniform.update(&cx, &v);

            cx.shed(|mut s| {
                let bg = Rgb::from_bytes([0; 3]);
                s.render(&redraw, bg).layer(&layer).set(&set).draw(&mesh);
            })
            .await;

            redraw.present();
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
                Some(winit::window::Fullscreen::Borderless(None))
            } else {
                None
            });
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.pressed(KeyCode::Escape);

    (render, resize, toggle_fullscreen, close, esc_pressed)
        .race()
        .await;

    Ok(())
}
