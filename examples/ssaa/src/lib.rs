use {
    dunge::{
        Options,
        buffer::{Filter, Sampler, Size, Texture, TextureSampler},
        color::Format,
        group::BoundTexture,
        sl_old::{Groups, Index, PassVertex, Render},
        store::Uniform,
        store_old::UniformOld,
    },
    dunge_winit::{Canvas, prelude::*},
    futures_concurrency::prelude::*,
    glam::{UVec2, Vec2, Vec4},
    std::{cell::RefCell, error, f32::consts, time::Duration},
    winit::keyboard::KeyCode,
};

type Error = Box<dyn error::Error>;

#[derive(Clone, Copy, Value)]
struct Vert {
    #[index]
    index: u32,
}

#[dunge(vertex)]
fn triangle_vs(v: Vert, offset: Uniform<f32>) -> Vec4 {
    let third = const { consts::TAU / 3. };
    let i = v.index as f32 * third + offset.read();
    Vec4::new(sl::cos(i), sl::sin(i), 0., 1.)
}

#[dunge(fragment)]
fn triangle_fs() -> Vec4 {
    Vec4::new(1., 0.4, 0.8, 1.)
}

#[derive(Clone, Copy, Value)]
struct Screen {
    pos: Vec2,
    uv: Vec2,
}

#[derive(Clone, Copy, Value)]
struct Io {
    #[position]
    pos: Vec4,
    uv: Vec2,
}

#[derive(Input)]
struct Map {
    texture: Texture,
    sampler: Sampler,
    offset: Uniform<Vec2>,
}

#[dunge(vertex)]
fn screen_vs(s: Screen) -> Io {
    Io {
        pos: sl::concat(s.pos, Vec2::new(0., 1.)),
        uv: s.uv,
    }
}

#[dunge(fragment)]
fn screen_fs(io: Io, m: Map) -> Vec4 {
    let offset = m.offset.read();
    let d0 = offset;
    let d1 = Vec2::new(offset.x, -offset.y);
    let d2 = Vec2::new(-offset.x, offset.y);
    let d3 = Vec2::new(-offset.x, -offset.y);
    let point = io.uv;

    (sl::texture_sample(m.texture.clone(), m.sampler.clone(), point + d0)
        + sl::texture_sample(m.texture.clone(), m.sampler.clone(), point + d1)
        + sl::texture_sample(m.texture.clone(), m.sampler.clone(), point + d2)
        + sl::texture_sample(m.texture, m.sampler, point + d3))
        * 0.25
}

pub async fn run(control: Control) -> Result<(), Error> {
    let triangle = |Index(idx): Index, Groups(offset): Groups<UniformOld<f32>>| {
        let color = Vec4::new(1., 0.4, 0.8, 1.);
        let third = const { consts::TAU / 3. };

        let i = sl_old::thunk(sl_old::f32(idx) * third + offset.load());
        Render {
            place: sl_old::vec4(sl_old::cos(i.clone()), sl_old::sin(i), 0., 1.),
            color,
        }
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct ScreenOld(Vec2, Vec2);

    #[derive(GroupLegacy)]
    struct MapOld {
        tex: BoundTexture,
        sam: TextureSampler,
        offset: UniformOld<Vec2>,
    }

    let screen = |PassVertex(v): PassVertex<ScreenOld>, Groups(map): Groups<MapOld>| Render {
        place: sl_old::vec4_concat(v.0, Vec2::new(0., 1.)),
        color: {
            let s = sl_old::thunk(sl_old::fragment(v.1));
            let tex = || map.tex.clone();
            let sam = || map.sam.clone();
            let offset = || map.offset.clone().load();
            let d0 = sl_old::vec2(offset().x(), offset().y());
            let d1 = sl_old::vec2(offset().x(), -offset().y());
            let d2 = sl_old::vec2(-offset().x(), offset().y());
            let d3 = sl_old::vec2(-offset().x(), -offset().y());
            (sl_old::texture_sample(tex(), sam(), s.clone() + d0)
                + sl_old::texture_sample(tex(), sam(), s.clone() + d1)
                + sl_old::texture_sample(tex(), sam(), s.clone() + d2)
                + sl_old::texture_sample(tex(), sam(), s + d3))
                * 0.25
        },
    };

    let cx = dunge::context().await?;
    let _shader2 = cx.make_shader(
        render! {
            groups: [Uniform<f32>],
            shaders: [triangle_vs, triangle_fs],
        }
        .inspect(|r| println!("{}", r.debug()))?,
    );

    let _screen_shader2 = cx.make_shader(
        render! {
            vertex: Screen,
            groups: [Map],
            shaders: [screen_vs, screen_fs],
        }
        .inspect(|r| println!("{}", r.debug()))?,
    );

    let shader = cx.make_shader_old(triangle);
    let screen_shader = cx.make_shader_old(screen);
    let offset = cx.make_uniform_old(&0.);
    let set = cx.make_set_old(&shader, &offset);

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time| {
        time += delta_time;
        let t = time.as_secs_f32() * 0.5;
        offset.update(&cx, &t);
    };

    const SCREEN_FACTOR: u32 = 2;

    let make_render_buffer = |size: UVec2| {
        let buffer_size = size.max(UVec2::ONE) * SCREEN_FACTOR;
        let buffer_size = Size::try_from(buffer_size).expect("non-zero size");
        let data = TextureData::empty(buffer_size, Format::SrgbAlpha)
            .render()
            .bind();

        RefCell::new(cx.make_texture(data))
    };

    let make_offset = |size: Size| {
        let screen_inv = const { 1. / SCREEN_FACTOR as f32 };
        screen_inv / size.as_uvec2().as_vec2()
    };

    let render_buffer = make_render_buffer(UVec2::ONE);
    let mut map = MapOld {
        tex: render_buffer.borrow().bind(),
        sam: cx.make_sampler(Filter::Nearest),
        offset: cx.make_uniform_old(&make_offset(render_buffer.borrow().size())),
    };

    let map_set = RefCell::new(cx.make_set_old(&screen_shader, &map));
    let handler = map_set.borrow().handler(&screen_shader);

    let screen_mesh = {
        const VERTS: [[ScreenOld; 4]; 1] = [[
            ScreenOld(Vec2::new(-1., -1.), Vec2::new(0., 1.)),
            ScreenOld(Vec2::new(1., -1.), Vec2::new(1., 1.)),
            ScreenOld(Vec2::new(1., 1.), Vec2::new(1., 0.)),
            ScreenOld(Vec2::new(-1., 1.), Vec2::new(0., 0.)),
        ]];

        let data = MeshData::from_quads(&VERTS)?;
        cx.make_mesh_old(&data)
    };

    let window = control
        .make_window(&cx)
        .with_title("ssaa")
        .with_canvas(Canvas::by_id("root"))
        .await?;

    let triangle_layer = cx.make_layer_old(&shader, render_buffer.borrow().format());
    let screen_layer = cx.make_layer_old(&screen_shader, window.format());

    let bg = window.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(redraw.delta_time());

            cx.shed(|s| {
                // draw the frame to the render buffer
                s.render(render_buffer.borrow(), bg)
                    .layer(&triangle_layer)
                    .set(&set)
                    .draw_points(3);

                // draw from the render buffer to the window
                s.render(&redraw, Options::default())
                    .layer(&screen_layer)
                    .set(map_set.borrow())
                    .draw(&screen_mesh);
            })
            .await;

            redraw.present();
        }
    };

    let update_render_buffer = async {
        loop {
            let size = window.resized().await;

            render_buffer.swap(&make_render_buffer(size));

            let buffer = render_buffer.borrow();

            map.tex = buffer.bind();
            map.offset.update(&cx, &make_offset(buffer.size()));

            cx.update_group(&mut map_set.borrow_mut(), &handler, &map);
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);
    (render, update_render_buffer, close, esc_pressed)
        .race()
        .await;

    Ok(())
}
