type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::window::WindowState) -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            glam::{Mat4, Quat, Vec3},
            prelude::*,
            sl::{Groups, InVertex, Out},
            uniform::Uniform,
        },
        std::cell::OnceCell,
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: [f32; 3],
        col: [f32; 3],
    }

    #[derive(Group)]
    struct Transform<'a>(&'a Uniform<[[f32; 4]; 4]>);

    let cube = |vert: InVertex<Vert>, Groups(tr): Groups<Transform>| Out {
        place: tr.0 * sl::vec4_with(vert.pos, 1.),
        color: sl::vec4_with(sl::fragment(vert.col), 1.),
    };

    let transform = |r, size| {
        let pos = Vec3::new(0., 0., -2.);
        let rot = Quat::from_rotation_y(r);
        let m = Mat4::from_rotation_translation(rot, pos);
        let p = {
            let (width, height) = size;
            let ratio = width as f32 / height as f32;
            Mat4::perspective_rh(1.6, ratio, 0.1, 100.)
        };

        p * m
    };

    let cx = dunge::context().await?;
    let cube_shader = cx.make_shader(cube);
    let mut r = 0.;
    let uniform = {
        let mat = transform(r, (1, 1));
        cx.make_uniform(mat)
    };

    let bind_transform = {
        let tr = Transform(&uniform);
        let mut binder = cx.make_binder(&cube_shader);
        binder.bind(&tr);
        binder.into_binding()
    };

    let mesh = {
        const P: f32 = 0.5;
        const VERTS: [Vert; 8] = [
            Vert {
                pos: [-P, -P, -P],
                col: [0., 0., 0.],
            },
            Vert {
                pos: [-P, -P, P],
                col: [0., 0., 1.],
            },
            Vert {
                pos: [-P, P, P],
                col: [0., 1., 1.],
            },
            Vert {
                pos: [-P, P, -P],
                col: [0., 1., 0.],
            },
            Vert {
                pos: [P, -P, -P],
                col: [1., 0., 0.],
            },
            Vert {
                pos: [P, P, -P],
                col: [1., 1., 0.],
            },
            Vert {
                pos: [P, P, P],
                col: [1., 1., 1.],
            },
            Vert {
                pos: [P, -P, P],
                col: [1., 0., 1.],
            },
        ];

        const INDXS: [[u16; 3]; 8] = [
            [0, 1, 2],
            [0, 2, 3], // -x
            [4, 5, 6],
            [4, 6, 7], // +x
            [0, 3, 5],
            [0, 5, 4], // -z
            [6, 2, 1],
            [7, 6, 1], // +z
        ];

        let data = MeshData::new(&VERTS, &INDXS)?;
        cx.make_mesh(&data)
    };

    let upd = {
        let cx = cx.clone();
        move |ctrl: &Control| {
            for key in ctrl.pressed_keys() {
                if key.code == KeyCode::Escape {
                    return Then::Close;
                }
            }

            r += ctrl.delta_time().as_secs_f32() * 0.5;
            let mat = transform(r, ctrl.size());
            uniform.update(&cx, mat);
            Then::Run
        }
    };

    let draw = {
        let cx = cx.clone();
        let layer = OnceCell::default();
        move |mut frame: Frame| {
            let layer = layer.get_or_init(|| cx.make_layer(&cube_shader, frame.format()));
            let opts = Rgba::from_standard([0.1, 0.05, 0.15, 1.]);
            frame.layer(&layer, opts).bind(&bind_transform).draw(&mesh);
        }
    };

    ws.run(cx, dunge::update(upd, draw))?;
    Ok(())
}
