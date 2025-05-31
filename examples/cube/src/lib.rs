use dunge_winit::prelude::*;

type Error = Box<dyn std::error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge_winit::{
            glam::{Mat4, Quat, Vec3},
            layer::{Config, Mode},
            sl::{Groups, InVertex, Render},
            storage::Uniform,
            winit::Canvas,
        },
        futures_concurrency::prelude::*,
        std::{cell::Cell, time::Duration},
        winit::{event::MouseButton, keyboard::KeyCode},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec3,
        col: Vec3,
    }

    let cube = |vert: InVertex<Vert>, Groups(m): Groups<Uniform<Mat4>>| Render {
        place: m * sl::vec4_with(vert.pos, 1.),
        color: sl::vec4_with(sl::fragment(vert.col), 1.),
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(cube);
    let transform = cx.make_uniform(&Mat4::IDENTITY);
    let set = cx.make_set(&shader, &transform);

    let mut time = Duration::ZERO;
    let mut update_scene = |(width, height), delta_time| {
        time += delta_time;

        let model = {
            let pos = Vec3::new(0., 0., -2.);
            let axis = Vec3::splat(1.).normalize();
            let angle = (time.as_secs_f32() * 2.).sin();
            let rot = Quat::from_axis_angle(axis, angle);
            Mat4::from_rotation_translation(rot, pos)
        };

        let projection = {
            let ratio = width as f32 / height as f32;
            Mat4::perspective_rh(1.6, ratio, 0.1, 100.)
        };

        let m = projection * model;
        transform.update(&cx, &m);
    };

    let mesh = {
        const VERTS: [Vert; 8] = {
            let p = 0.5;

            [
                Vert {
                    pos: Vec3::new(-p, -p, -p),
                    col: Vec3::new(0., 0., 0.),
                },
                Vert {
                    pos: Vec3::new(-p, -p, p),
                    col: Vec3::new(0., 0., 1.),
                },
                Vert {
                    pos: Vec3::new(-p, p, p),
                    col: Vec3::new(0., 1., 1.),
                },
                Vert {
                    pos: Vec3::new(-p, p, -p),
                    col: Vec3::new(0., 1., 0.),
                },
                Vert {
                    pos: Vec3::new(p, -p, -p),
                    col: Vec3::new(1., 0., 0.),
                },
                Vert {
                    pos: Vec3::new(p, p, -p),
                    col: Vec3::new(1., 1., 0.),
                },
                Vert {
                    pos: Vec3::new(p, p, p),
                    col: Vec3::new(1., 1., 1.),
                },
                Vert {
                    pos: Vec3::new(p, -p, p),
                    col: Vec3::new(1., 0., 1.),
                },
            ]
        };

        const INDXS: [[u32; 3]; 12] = [
            [0, 1, 2],
            [0, 2, 3], // -x
            [4, 5, 6],
            [4, 6, 7], // +x
            [0, 4, 7],
            [0, 7, 1], // -y
            [3, 2, 6],
            [3, 6, 5], // +y
            [0, 3, 5],
            [0, 5, 4], // -z
            [6, 2, 1],
            [7, 6, 1], // +z
        ];

        let data = MeshData::new(&VERTS, &INDXS)?;
        cx.make_mesh(&data)
    };

    let window = {
        let attr = Attributes::default()
            .with_title("cube")
            .with_canvas(Canvas::by_id("root"));

        control.make_window(&cx, attr).await?
    };

    let mouse = async {
        loop {
            window.button_pressed(MouseButton::Left).await;
            let Some(p) = window.cursor_position() else {
                continue;
            };

            println!("pressed at {p}");

            window.button_released(MouseButton::Left).await;
            println!("released");
        }
    };

    let conf = Config::from(window.format());
    let layer_solid = cx.make_layer(&shader, conf.clone());
    let layer_wireframe = cx.make_layer(
        &shader,
        Config {
            mode: Mode::Line,
            ..conf
        },
    );

    let mode = Cell::new(true);

    let bg = window.format().rgb_from_bytes([25, 10, 40]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(window.size(), redraw.delta_time());

            let layer = if mode.get() {
                &layer_solid
            } else {
                &layer_wireframe
            };

            cx.shed(|mut s| {
                s.render(&redraw, bg).layer(layer).set(&set).draw(&mesh);
            })
            .await;

            redraw.present();
        }
    };

    let toggle_mode = async {
        loop {
            window.key_pressed(KeyCode::KeyT).await;
            mode.set(!mode.get());
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);
    (mouse, render, toggle_mode, close, esc_pressed)
        .race()
        .await;

    Ok(())
}
