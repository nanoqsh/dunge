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
        step: &'a Uniform<[f32; 2]>,
        // TODO: fix frame size alignment
    }

    let screen = |vert: InVertex<Screen>, Groups(map): Groups<Map>| Out {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: {
            let [st] = sl::thunk(sl::fragment(vert.1));
            let d1 = sl::vec2(0., map.step.clone().y());
            let d2 = sl::vec2(map.step.clone().x(), 0.);
            let d3 = sl::vec2(map.step.clone().x(), map.step.y());
            (sl::texture_sample(map.tex.clone(), map.sam.clone(), st.clone())
                + sl::texture_sample(map.tex.clone(), map.sam.clone(), st.clone() + d1)
                + sl::texture_sample(map.tex.clone(), map.sam.clone(), st.clone() + d2)
                + sl::texture_sample(map.tex, map.sam, st + d3))
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

    let make_render_buf = |cx: &Context, (width, height)| {
        let size = (u32::max(width * 2, 1), u32::max(height * 2, 1));
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

    let step_value = |size| <[u32; 2]>::from(size).map(|v| 0.5 / v as f32);
    let step = cx.make_uniform(step_value(render_buf.size()));

    let (bind_map, handler) = {
        let map = Map {
            tex: BoundTexture::new(render_buf.color()),
            sam: &sam,
            step: &step,
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
            let v = step_value(state.render_buf.size());
            step.update(&cx, v);

            let map = Map {
                tex: BoundTexture::new(state.render_buf.color()),
                sam: &sam,
                step: &step,
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
