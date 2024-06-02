type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::window::WindowState) -> Result<(), Error> {
    use {
        dunge::{
            bind::UniqueBinding,
            color::Rgba,
            glam::{Vec2, Vec4},
            group::BoundTexture,
            prelude::*,
            sl::{Groups, InVertex, Index, Out},
            texture::{DrawTexture, Filter, Sampler},
            uniform::Uniform,
            Format,
        },
        std::{cell::OnceCell, f32::consts},
    };

    const COLOR: Vec4 = Vec4::new(1., 0.4, 0.8, 1.);
    const THIRD: f32 = consts::TAU / 3.;
    const SCREEN_FACTOR: u32 = 2;

    #[derive(Group)]
    struct Offset<'a>(&'a Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset>| {
        let i = sl::thunk(sl::f32(idx) * THIRD + offset.0);
        Out {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color: COLOR,
        }
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
            let s = sl::thunk(sl::fragment(vert.1));
            let tex = || map.tex.clone();
            let sam = || map.sam.clone();
            let stp = || map.stp.clone();
            let d0 = sl::vec2(stp().x(), stp().y());
            let d1 = sl::vec2(stp().x(), -stp().y());
            let d2 = sl::vec2(-stp().x(), stp().y());
            let d3 = sl::vec2(-stp().x(), -stp().y());
            (sl::texture_sample(tex(), sam(), s.clone() + d0)
                + sl::texture_sample(tex(), sam(), s.clone() + d1)
                + sl::texture_sample(tex(), sam(), s.clone() + d2)
                + sl::texture_sample(tex(), sam(), s + d3))
                * 0.25
        },
    };

    let cx = dunge::context().await?;
    let triangle_shader = cx.make_shader(triangle);
    let screen_shader = cx.make_shader(screen);
    let mut r = 0.;
    let uniform = cx.make_uniform(r);
    let bind = {
        let offset = Offset(&uniform);
        let mut binder = cx.make_binder(&triangle_shader);
        binder.bind(&offset);
        binder.into_binding()
    };

    let make_render_buf = |cx: &Context, (width, height)| {
        let size = (
            u32::max(width, 1) * SCREEN_FACTOR,
            u32::max(height, 1) * SCREEN_FACTOR,
        );

        let data = TextureData::empty(size, Format::SrgbAlpha)
            .expect("non-zero size")
            .with_draw()
            .with_bind();

        cx.make_texture(data)
    };

    let render_buf = make_render_buf(&cx, (1, 1));
    let sam = cx.make_sampler(Filter::Nearest);

    let make_stp = |size| {
        const SCREEN_INV: f32 = 1. / SCREEN_FACTOR as f32;

        <[u32; 2]>::from(size).map(|v| SCREEN_INV / v as f32)
    };

    let buf_size = render_buf.draw_texture().size();
    let stp = cx.make_uniform(make_stp(buf_size));
    let (bind_map, handler) = {
        let map = Map {
            tex: BoundTexture::new(&render_buf),
            sam: &sam,
            stp: &stp,
        };

        let mut binder = cx.make_binder(&screen_shader);
        let handler = binder.bind(&map);
        (binder.into_binding(), handler)
    };

    let upd = move |state: &mut State<_>, ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        if let Some(size) = ctrl.resized() {
            state.render_buf = make_render_buf(&state.cx, size);
            let buf_size = state.render_buf.draw_texture().size();
            stp.update(&state.cx, make_stp(buf_size));
            let map = Map {
                tex: BoundTexture::new(&state.render_buf),
                sam: &sam,
                stp: &stp,
            };

            dunge::then!(state.cx.update_group(&mut state.bind_map, &handler, &map));
        }

        r += ctrl.delta_time().as_secs_f32() * 0.5;
        uniform.update(&state.cx, r);
        Then::Run
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

    let triangle_layer = cx.make_layer(&triangle_shader, Format::SrgbAlpha);
    let screen_layer = OnceCell::default();
    let draw = {
        let cx = cx.clone();
        move |state: &State<_>, mut frame: Frame| {
            let main = |mut frame: Frame| {
                let opts = Rgba::from_standard([0.1, 0.05, 0.15, 1.]);
                frame
                    .layer(&triangle_layer, opts)
                    .bind(&bind)
                    .draw_points(3);
            };

            state.cx.draw_to(&state.render_buf, dunge::draw(main));

            let screen_layer =
                screen_layer.get_or_init(|| cx.make_layer(&screen_shader, frame.format()));

            frame
                .layer(screen_layer, Options::default())
                .bind(&state.bind_map)
                .draw(&screen_mesh);
        }
    };

    struct State<R> {
        cx: Context,
        render_buf: R,
        bind_map: UniqueBinding,
    }

    let state = State {
        cx: cx.clone(),
        render_buf,
        bind_map,
    };

    ws.run(cx, dunge::update_with_state(state, upd, draw))?;
    Ok(())
}
