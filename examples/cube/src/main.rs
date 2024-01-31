type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(err) = helpers::block_on(run()) {
        eprintln!("error: {err}");
    }
}

async fn run() -> Result<(), Error> {
    use dunge::{
        bind::UniqueBinding,
        color::Rgba,
        glam::{Mat4, Quat, Vec2, Vec3},
        group::BoundTexture,
        prelude::*,
        sl::{Groups, InVertex, Out},
        texture::{self, Filter, Sampler, Texture, ZeroSized},
        uniform::Uniform,
        Format, Options,
    };

    type RenderTexture = texture::Draw<texture::Bind<Texture>>;

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

    #[repr(C)]
    #[derive(Vertex)]
    struct Screen([f32; 2], [f32; 2]);

    #[derive(Group)]
    struct Map<'a> {
        tex: BoundTexture<'a>,
        sam: &'a Sampler,
    }

    let screen = |vert: InVertex<Screen>, Groups(map): Groups<Map>| Out {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: sl::texture_sample(map.tex, map.sam, sl::fragment(vert.1)),
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
    let cube_shader = cx.make_shader(cube);
    let screen_shader = cx.make_shader(screen);
    let mut r = 0.;
    let uniform = {
        let mat = transform(r, window.size());
        cx.make_uniform(mat)
    };

    let bind_transform = {
        let tr = Transform(&uniform);
        let mut binder = cx.make_binder(&cube_shader);
        binder.bind(&tr);
        binder.into_binding()
    };

    let make_screen_tex = |cx: &Context, size| -> Result<_, ZeroSized> {
        use dunge::texture::Data;

        let data = Data::empty(size, Format::RgbAlpha)?.with_bind().with_draw();
        Ok(cx.make_texture(data))
    };

    let mut tex = make_screen_tex(&cx, window.size())?;
    let sam = cx.make_sampler(Filter::Nearest);
    let (bind_map, handler) = {
        let map = Map {
            tex: BoundTexture::new(&tex),
            sam: &sam,
        };

        let mut binder = cx.make_binder(&screen_shader);
        let handler = binder.bind(&map);
        (binder.into_binding(), handler)
    };

    let mesh = {
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

    let screen_mesh = {
        use dunge::mesh::Data;

        const VERTS: [Screen; 4] = [
            Screen([-1., -1.], [0., 1.]),
            Screen([1., -1.], [1., 1.]),
            Screen([1., 1.], [1., 0.]),
            Screen([-1., 1.], [0., 0.]),
        ];

        let data = Data::from_quads(&[VERTS])?;
        cx.make_mesh(&data)
    };

    let main_layer = cx.make_layer(&cube_shader, Format::RgbAlpha);
    let screen_layer = cx.make_layer(&screen_shader, window.format());
    let mut size = window.size();
    let upd = |state: &mut State, ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        if size != ctrl.size() {
            size = ctrl.size();
            *state.tex = dunge::then_try! { make_screen_tex(&cx, size) };
            let map = Map {
                tex: BoundTexture::new(state.tex),
                sam: &sam,
            };

            dunge::then_try! {
                cx.update_group(&mut state.bind_map, &handler, &map);
            }
        }

        r += ctrl.delta_time().as_secs_f32();
        let mat = transform(r, ctrl.size());
        uniform.update(&cx, mat);
        Then::Run
    };

    let clear = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = |state: &State, mut frame: Frame| {
        let main = |mut frame: Frame| {
            frame
                .layer(&main_layer, clear)
                .bind(&bind_transform)
                .draw(&mesh);
        };

        cx.draw_to(state.tex, draw::from_fn(main));

        frame
            .layer(&screen_layer, Options::default())
            .bind(&state.bind_map)
            .draw(&screen_mesh);
    };

    struct State<'a> {
        tex: &'a mut RenderTexture,
        bind_map: UniqueBinding,
    }

    let state = State {
        tex: &mut tex,
        bind_map,
    };

    let handle = update::with_state(state, upd, draw);
    window.run(handle).map_err(Box::from)
}
