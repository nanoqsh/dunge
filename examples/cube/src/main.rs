type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(err) = helpers::block_on(run()) {
        eprintln!("error: {err}");
    }
}

async fn run() -> Result<(), Error> {
    use dunge::{
        color::Rgba,
        el::{KeyCode, Then},
        glam::{Mat4, Quat, Vec3},
        sl::{self, Groups, InVertex, Out},
        uniform::Uniform,
        update, Control, Frame, Group, Vertex,
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: [f32; 3],
        col: [f32; 3],
    }

    type Mat = [[f32; 4]; 4];

    #[derive(Group)]
    struct Transform<'a>(&'a Uniform<Mat>);

    let cube = |vert: InVertex<Vert>, Groups(tr): Groups<Transform>| Out {
        place: tr.0 * sl::vec4_with(vert.pos, 1.),
        color: sl::vec4_with(sl::fragment(vert.col), 1.),
    };

    let window = dunge::window().with_title("Cube").await?;
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

    let cx = window.context();
    let shader = cx.make_shader(cube);
    let mut r = 0.;
    let uniform = {
        let mat = transform(r, window.size());
        // TODO: `IntoValue` trait
        cx.make_uniform(mat.to_cols_array_2d())
    };

    let bind = {
        let mut binder = cx.make_binder(&shader);
        let tr = Transform(&uniform);
        binder.bind(&tr);
        binder.into_binding()
    };

    let mech = {
        use dunge::mesh::Data;

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

        let data = Data::new(&VERTS, &INDXS)?;
        cx.make_mesh(&data)
    };

    let layer = cx.make_layer(&shader, window.format());
    let update = |ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        r += ctrl.delta_time().as_secs_f32();
        let mat = transform(r, ctrl.size());
        uniform.update(&cx, mat.to_cols_array_2d());
        Then::Run
    };

    let clear = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = |mut frame: Frame| {
        frame.layer(&layer, clear).bind(&bind).draw(&mech);
    };

    window.run(update::from_fn(update, draw))?;
    Ok(())
}
