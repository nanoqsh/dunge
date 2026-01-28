use {
    dunge::store::Uniform,
    dunge_winit::{Canvas, prelude::*},
    futures_concurrency::prelude::*,
    glam::{Mat4, Quat, UVec2, Vec3, Vec4},
    std::{error, time::Duration},
    winit::{event::MouseButton, keyboard::KeyCode},
};

#[derive(Clone, Copy, Value, Bytes)]
struct Vert {
    pos: Vec3,
    col: Vec3,
}

#[derive(Clone, Copy, Value)]
struct Io {
    #[position]
    pos: Vec4,
    col: Vec3,
}

#[dunge(vertex)]
fn vs(v: Vert, m: Uniform<Mat4>) -> Io {
    let pos = m.read() * sl::append(v.pos, 1.);
    Io { pos, col: v.col }
}

#[dunge(fragment)]
fn fs(io: Io) -> Vec4 {
    sl::append(io.col, 1.)
}

type Error = Box<dyn error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    let cx = dunge::context().await?;
    let shader = cx.make_shader(render! {
        vertex: Vert,
        groups: [Uniform<Mat4>],
        shaders: [vs, fs],
    }?);

    let transform = cx.make_uniform2(&Mat4::IDENTITY);
    let set = cx.make_set2(&shader, &transform);

    let mut time = Duration::ZERO;
    let mut update = |size: UVec2, delta_time| {
        time += delta_time;

        let model = {
            let pos = Vec3::new(0., 0., -2.);
            let axis = Vec3::splat(1.).normalize();
            let angle = f32::sin(time.as_secs_f32() * 2.);
            let rot = Quat::from_axis_angle(axis, angle);
            Mat4::from_rotation_translation(rot, pos)
        };

        let projection = {
            let ratio = size.x as f32 / size.y as f32;
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
        cx.make_mesh2(&data)
    };

    let window = control
        .make_window(&cx)
        .with_title("cube")
        .with_canvas(Canvas::by_id("root"))
        .await?;

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

    let layer = cx.make_layer2(&shader, window.format());

    let bg = window.format().rgb_from_bytes([25, 10, 40]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update(window.size(), redraw.delta_time());

            cx.shed(|s| {
                s.render(&redraw, bg).layer(&layer).set(&set).draw(&mesh);
            })
            .await;

            redraw.present();
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);
    (mouse, render, close, esc_pressed).race().await;

    Ok(())
}
