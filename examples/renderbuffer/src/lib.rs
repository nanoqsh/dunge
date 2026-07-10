use {
    dunge::{
        Options, RenderBuffer, RenderShader,
        buffer::{Filter, Sampler, Texture},
        color::Format,
        mesh,
    },
    dunge_winit::{Canvas, prelude::*},
    futures_concurrency::prelude::*,
    glam::{UVec2, Vec2, Vec4},
    std::{cell::RefCell, error, num::NonZeroU32},
    winit::keyboard::KeyCode,
};

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

#[dunge(vertex)]
fn screen_vs(v: Screen) -> Io {
    let pos = sl::concat(v.pos, Vec2::new(0., 1.));
    Io { pos, uv: v.uv }
}

#[derive(Input)]
struct Frame {
    tex: Texture,
    sam: Sampler,
}

#[dunge(fragment)]
fn screen_fs(io: Io, f: Frame) -> Vec4 {
    sl::texture_sample(f.tex, f.sam, io.uv)
}

type ScreenSet = (Frame,);
type ScreenShader = RenderShader<ScreenSet, Screen>;

fn screen(cx: &Context) -> ScreenShader {
    cx.make_shader(
        dunge::render! {
            vertex: Screen,
            groups: [Frame],
            shaders: [screen_vs, screen_fs],
        }
        .expect("typecheck"),
    )
}

type Error = Box<dyn error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    let cx = dunge::context().await?;
    let window = control
        .make_window(&cx)
        .with_title("renderbuffer")
        .with_canvas(Canvas::by_id("root"))
        .await?;

    let screen_shader = screen(&cx);
    let screen_mesh = cx.make_mesh(&MeshData::from_quads(&[mesh::SCREEN.map(
        |(x, y, u, v)| Screen {
            pos: Vec2::new(x, y),
            uv: Vec2::new(u, v),
        },
    )])?);

    let make_render_buffer = |size: UVec2| {
        let width = NonZeroU32::new(size.x).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(size.y).unwrap_or(NonZeroU32::MIN);

        let data = TextureData::empty((width, height), window.format())
            .render()
            .bind();

        let color = cx.make_texture(data);

        let data = TextureData::empty((width, height), Format::Depth).render();
        let depth = cx.make_texture(data);
        RefCell::new(RenderBuffer::new(color, depth))
    };

    let render_buffer = make_render_buffer(UVec2::ONE);
    let mut frame = Frame {
        tex: render_buffer.borrow().color().texture(),
        sam: cx.make_sampler(Filter::Nearest).sampler(),
    };

    let screen_set = RefCell::new(cx.make_set(&screen_shader, &frame));
    let screen_handler = screen_set.borrow_mut().handler(&screen_shader);
    let window_layer = cx.make_layer(&screen_shader, window.format());

    let mut get_render_buffer = |size: UVec2| {
        if let buffer = render_buffer.borrow()
            && buffer.size().as_uvec2() == size
        {
            buffer
        } else {
            render_buffer.swap(&make_render_buffer(size));
            let buffer = render_buffer.borrow();
            frame.tex = buffer.color().texture();
            cx.update_group(&mut screen_set.borrow_mut(), &screen_handler, &frame);
            buffer
        }
    };

    let bg = window.format().rgb_from_bytes([25, 10, 40]);
    let render = async {
        loop {
            let redraw = window.redraw().await;

            cx.shed(|s| {
                let render_buffer = get_render_buffer(window.size());

                // Render something to the buffer
                _ = s.render(render_buffer, bg);

                s.render(&redraw, Options::default())
                    .layer(&window_layer)
                    .set(screen_set.borrow())
                    .draw(&screen_mesh);
            })
            .await;

            redraw.present();
        }
    };

    let close = window.close_requested();
    let esc_pressed = window.key_pressed(KeyCode::Escape);
    (render, close, esc_pressed).race().await;

    Ok(())
}
