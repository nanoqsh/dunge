use {
    dunge::{
        Options,
        buffer::{Filter, Sampler, Size, Texture},
        color::Format,
        mesh,
        store::Uniform,
    },
    dunge_winit::{Canvas, prelude::*},
    futures_concurrency::prelude::*,
    glam::{UVec2, Vec2, Vec4},
    std::{cell::RefCell, error, time::Duration},
    triangle::shader,
    winit::keyboard::KeyCode,
};

type Error = Box<dyn error::Error>;

#[derive(Clone, Copy, Value, Bytes)]
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
    (sl::texture_sample(m.texture.clone(), m.sampler.clone(), io.uv + d0)
        + sl::texture_sample(m.texture.clone(), m.sampler.clone(), io.uv + d1)
        + sl::texture_sample(m.texture.clone(), m.sampler.clone(), io.uv + d2)
        + sl::texture_sample(m.texture, m.sampler, io.uv + d3))
        * 0.25
}

pub async fn run(control: Control) -> Result<(), Error> {
    let cx = dunge::context().await?;
    let triangle = cx.make_shader(render! {
        groups: [Uniform<f32>],
        shaders: [shader::vs, shader::fs],
    }?);

    let screen = cx.make_shader(render! {
        vertex: Screen,
        groups: [Map],
        shaders: [screen_vs, screen_fs],
    }?);

    let offset = cx.make_uniform(&0.);
    let set = cx.make_set(&triangle, &offset);

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time| {
        time += delta_time;
        let t = time.as_secs_f32() * 0.5;
        offset.update(&cx, &t);
    };

    const SCREEN_FACTOR: u32 = 2;

    let make_render_buffer = |size: UVec2| {
        let buffer_size = size.max(UVec2::ONE) * SCREEN_FACTOR;
        let buffer_size = Size::from_uvec2(buffer_size);
        let data = TextureData::empty(buffer_size, Format::SrgbAlpha)
            .render()
            .bind();

        cx.make_texture(data)
    };

    let make_offset = |size: Size| {
        let screen_inv = const { 1. / SCREEN_FACTOR as f32 };
        screen_inv / size.as_uvec2().as_vec2()
    };

    let (mut map, render_buffer) = {
        let buffer = make_render_buffer(UVec2::ONE);

        (
            Map {
                texture: buffer.texture(),
                sampler: cx.make_sampler(Filter::Nearest).sampler(),
                offset: cx.make_uniform(&make_offset(buffer.size())),
            },
            RefCell::new(buffer),
        )
    };

    let map_set = RefCell::new(cx.make_set(&screen, &map));
    let handler = map_set.borrow().handler(&screen);

    let screen_mesh = {
        let data = &mesh::SCREEN.map(|(x, y, u, v)| Screen {
            pos: Vec2::new(x, y),
            uv: Vec2::new(u, v),
        });

        cx.make_mesh(&data.into())
    };

    let window = control
        .make_window(&cx)
        .with_title("ssaa")
        .with_canvas(Canvas::by_id("root"))
        .await?;

    let triangle_layer = cx.make_layer(&triangle, render_buffer.borrow().format());
    let screen_layer = cx.make_layer(&screen, window.format());

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

            render_buffer.swap(&make_render_buffer(size).into());

            let buffer = render_buffer.borrow();
            map.texture = buffer.texture();
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
