type Error = Box<dyn std::error::Error>;

pub async fn run(ws: dunge::window::WindowState) -> Result<(), Error> {
    use dunge::{
        bind::UniqueBinding,
        color::Rgba,
        glam::{Vec2, Vec4},
        group::BoundTexture,
        prelude::*,
        sl::{Groups, InVertex, Index, Render},
        texture::{DrawTexture, Filter, Sampler},
        uniform::Uniform,
        Format,
    };

    const SCREEN_FACTOR: u32 = 2;

    #[derive(Group)]
    struct Offset<'a>(&'a Uniform<f32>);

    let triangle = |Index(idx): Index, Groups(offset): Groups<Offset>| {
        use std::f32::consts;

        let color = const { Vec4::new(1., 0.4, 0.8, 1.) };
        let third = const { consts::TAU / 3. };

        let i = sl::thunk(sl::f32(idx) * third + offset.0);
        Render {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color,
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

    let screen = |vert: InVertex<Screen>, Groups(map): Groups<Map>| Render {
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
        binder.add(&offset);
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

    let make_stp = |(width, height)| {
        let screen_inv = const { 1. / SCREEN_FACTOR as f32 };
        [screen_inv / width as f32, screen_inv / height as f32]
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
        let handler = binder.add(&map);
        (binder.into_binding(), handler)
    };

    let screen_mesh = {
        let verts = const {
            [[
                Screen([-1., -1.], [0., 1.]),
                Screen([1., -1.], [1., 1.]),
                Screen([1., 1.], [1., 0.]),
                Screen([-1., 1.], [0., 0.]),
            ]]
        };

        let data = MeshData::from_quads(&verts)?;
        cx.make_mesh(&data)
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

    let make_handler = move |cx: &Context, view: &View| {
        let triangle_layer = cx.make_layer(&triangle_shader, Format::SrgbAlpha);
        let screen_layer = cx.make_layer(&screen_shader, view.format());

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

        let draw = move |state: &State<_>, mut frame: Frame| {
            let main = |mut frame: Frame| {
                let opts = Rgba::from_standard([0.1, 0.05, 0.15, 1.]);
                frame
                    .layer(&triangle_layer, opts)
                    .bind(&bind)
                    .draw_points(3);
            };

            state.cx.draw_to(&state.render_buf, dunge::draw(main));

            frame
                .layer(&screen_layer, Options::default())
                .bind(&state.bind_map)
                .draw(&screen_mesh);
        };

        dunge::update_with_state(state, upd, draw)
    };

    ws.run(cx, dunge::make(make_handler))?;
    Ok(())
}
