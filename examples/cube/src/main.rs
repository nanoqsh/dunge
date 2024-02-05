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
        layer::Config,
        mesh::MeshData,
        prelude::*,
        sl::{Groups, InVertex, Out},
        texture::{Filter, Sampler, TextureData},
        uniform::Uniform,
        Format, RenderBuffer,
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

    #[repr(C)]
    #[derive(Vertex)]
    struct Screen([f32; 2], [f32; 2]);

    #[derive(Group)]
    struct Map<'a> {
        tex: BoundTexture<'a>,
        sam: &'a Sampler,
        stp: &'a Uniform<[f32; 2]>,
    }

    let screen = |vert: InVertex<Screen>, Groups(map): Groups<Map>| Out {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: {
            let [s0, s1, s2, s3] = sl::thunk(sl::fragment(vert.1));
            let tex = || map.tex.clone();
            let sam = || map.sam.clone();
            let stp = || map.stp.clone();
            let d0 = sl::vec2(stp().x(), stp().y());
            let d1 = sl::vec2(stp().x(), -stp().y());
            let d2 = sl::vec2(-stp().x(), stp().y());
            let d3 = sl::vec2(-stp().x(), -stp().y());
            (sl::texture_sample(tex(), sam(), s0 + d0)
                + sl::texture_sample(tex(), sam(), s1 + d1)
                + sl::texture_sample(tex(), sam(), s2 + d2)
                + sl::texture_sample(tex(), sam(), s3 + d3))
                * 0.25
        },
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

    const SCREEN_FACTOR: u32 = 2;

    let make_render_buf = |cx: &Context, (width, height)| {
        let size = (
            u32::max(width, 1) * SCREEN_FACTOR,
            u32::max(height, 1) * SCREEN_FACTOR,
        );

        let color = {
            let data = TextureData::empty(size, Format::RgbAlpha)
                .expect("non-zero size")
                .with_draw()
                .with_bind();

            cx.make_texture(data)
        };

        let depth = {
            let data = TextureData::empty(size, Format::Depth)
                .expect("non-zero size")
                .with_draw();

            cx.make_texture(data)
        };

        RenderBuffer::new(color, depth)
    };

    let mut render_buf = make_render_buf(&cx, window.size());
    let sam = cx.make_sampler(Filter::Nearest);

    let make_stp = |size| {
        const SCREEN_INV: f32 = 1. / SCREEN_FACTOR as f32;

        <[u32; 2]>::from(size).map(|v| SCREEN_INV / v as f32)
    };

    let stp = cx.make_uniform(make_stp(render_buf.size()));

    let (bind_map, handler) = {
        let map = Map {
            tex: BoundTexture::new(render_buf.color()),
            sam: &sam,
            stp: &stp,
        };

        let mut binder = cx.make_binder(&screen_shader);
        let handler = binder.bind(&map);
        (binder.into_binding(), handler)
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

    let screen_mesh = {
        const VERTS: [Screen; 4] = [
            Screen([-1., -1.], [0., 1.]),
            Screen([1., -1.], [1., 1.]),
            Screen([1., 1.], [1., 0.]),
            Screen([-1., 1.], [0., 0.]),
        ];

        let data = MeshData::from_quads(&[VERTS])?;
        cx.make_mesh(&data)
    };

    let main_layer = {
        let conf = Config {
            depth: true,
            ..Default::default()
        };

        cx.make_layer(&cube_shader, conf)
    };

    let screen_layer = cx.make_layer(&screen_shader, window.format());
    let upd = |state: &mut State<_>, ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        if let Some(size) = ctrl.resized() {
            *state.render_buf = make_render_buf(&cx, size);
            stp.update(&cx, make_stp(state.render_buf.size()));

            let map = Map {
                tex: BoundTexture::new(state.render_buf.color()),
                sam: &sam,
                stp: &stp,
            };

            dunge::then!(cx.update_group(&mut state.bind_map, &handler, &map));
        }

        r += ctrl.delta_time().as_secs_f32() * 0.5;
        let mat = transform(r, ctrl.size());
        uniform.update(&cx, mat);
        Then::Run
    };

    let draw = |state: &State<_>, mut frame: Frame| {
        let main = |mut frame: Frame| {
            let opts = Options::default()
                .clear_color(Rgba::from_standard([0.1, 0.05, 0.15, 1.]))
                .clear_depth(1.);

            frame
                .layer(&main_layer, opts)
                .bind(&bind_transform)
                .draw(&mesh);
        };

        cx.draw_to(state.render_buf, dunge::draw(main));

        frame
            .layer(&screen_layer, Options::default())
            .bind(&state.bind_map)
            .draw(&screen_mesh);
    };

    struct State<'a, R> {
        render_buf: &'a mut R,
        bind_map: UniqueBinding,
    }

    let state = State {
        render_buf: &mut render_buf,
        bind_map,
    };

    let handle = dunge::update_with(state, upd, draw);
    window.run(handle).map_err(Box::from)
}
