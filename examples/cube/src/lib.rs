use dunge_winit::{Context, runtime::Control};

type Error = Box<dyn std::error::Error>;

pub async fn run(cx: Context, control: Control) -> Result<(), Error> {
    use {
        dunge_winit::{
            color::Rgb,
            glam::{Mat4, Quat, Vec3},
            prelude::*,
            runtime::Attributes,
            sl::{Groups, InVertex, Render},
            uniform::Uniform,
            winit::keyboard::KeyCode,
        },
        futures_lite::future,
        std::time::Duration,
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec3,
        col: Vec3,
    }

    #[derive(Group)]
    struct Transform<'uni>(&'uni Uniform<Mat4>);

    let cube = |vert: InVertex<Vert>, Groups(tr): Groups<Transform<'_>>| Render {
        place: tr.0 * sl::vec4_with(vert.pos, 1.),
        color: sl::vec4_with(sl::fragment(vert.col), 1.),
    };

    let shader = cx.make_shader(cube);
    let uniform = cx.make_uniform(&Mat4::IDENTITY);
    let transform = cx.make_set(&shader, Transform(&uniform));

    let mut r = 0.;
    let mut update_scene = |(width, height), delta_time: Duration| {
        r += delta_time.as_secs_f32() * 1.5;

        let model = {
            let pos = Vec3::new(0., 0., -2.);
            let rot = Quat::from_axis_angle(Vec3::splat(1.).normalize(), f32::sin(r));
            Mat4::from_rotation_translation(rot, pos)
        };

        let projection = {
            let ratio = width as f32 / height as f32;
            Mat4::perspective_rh(1.6, ratio, 0.1, 100.)
        };

        let mat = projection * model;
        uniform.update(&cx, &mat);
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

        const INDXS: [[u16; 3]; 12] = [
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

    let window = control.make_window(Attributes::default()).await?;
    let layer = cx.make_layer(&shader, window.format());

    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(window.size(), redraw.delta_time());

            cx.shed(|mut s| {
                let bg = Rgb::from_standard([0.1, 0.05, 0.15]);
                s.render(&redraw, bg)
                    .layer(&layer)
                    .set(&transform)
                    .draw(&mesh);
            })
            .await;

            redraw.present();
        }
    };

    let close = window.pressed(KeyCode::Escape);
    future::or(render, close).await;

    Ok(())
}
