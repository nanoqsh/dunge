use dunge_winit::prelude::*;

type Error = Box<dyn std::error::Error>;

pub async fn run(control: Control) -> Result<(), Error> {
    use {
        dunge_winit::{
            Options,
            buffer::{Filter, Format, Sampler, Size},
            glam::{Vec2, Vec4},
            group::BoundTexture,
            sl::{Groups, InVertex, Index, Render},
            storage::Uniform,
            winit::Canvas,
        },
        futures_concurrency::prelude::*,
        std::{cell::RefCell, f32::consts, time::Duration},
        winit::keyboard::KeyCode,
    };

    const SCREEN_FACTOR: u32 = 2;

    let triangle = |Index(idx): Index, Groups(offset): Groups<Uniform<f32>>| {
        let color = Vec4::new(1., 0.4, 0.8, 1.);
        let third = const { consts::TAU / 3. };

        let i = sl::thunk(sl::f32(idx) * third + offset);
        Render {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i), 0., 1.),
            color,
        }
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Screen(Vec2, Vec2);

    #[derive(Group)]
    struct Map {
        tex: BoundTexture,
        sam: Sampler,
        offset: Uniform<Vec2>,
    }

    let screen = |vert: InVertex<Screen>, Groups(map): Groups<Map>| Render {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: {
            let s = sl::thunk(sl::fragment(vert.1));
            let tex = || map.tex.clone();
            let sam = || map.sam.clone();
            let offset = || map.offset.clone();
            let d0 = sl::vec2(offset().x(), offset().y());
            let d1 = sl::vec2(offset().x(), -offset().y());
            let d2 = sl::vec2(-offset().x(), offset().y());
            let d3 = sl::vec2(-offset().x(), -offset().y());
            (sl::texture_sample(tex(), sam(), s.clone() + d0)
                + sl::texture_sample(tex(), sam(), s.clone() + d1)
                + sl::texture_sample(tex(), sam(), s.clone() + d2)
                + sl::texture_sample(tex(), sam(), s + d3))
                * 0.25
        },
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(triangle);
    let screen_shader = cx.make_shader(screen);
    let offset = cx.make_uniform(&0.);
    let set = cx.make_set(&shader, &offset);

    let mut time = Duration::ZERO;
    let mut update_scene = |delta_time| {
        time += delta_time;
        let t = time.as_secs_f32() * 0.5;
        offset.update(&cx, &t);
    };

    let make_render_buffer = |(width, height)| {
        let size = (
            u32::max(width, 1) * SCREEN_FACTOR,
            u32::max(height, 1) * SCREEN_FACTOR,
        );

        let size = Size::try_from(size).expect("non-zero size");
        let data = TextureData::empty(size, Format::SrgbAlpha).render().bind();
        RefCell::new(cx.make_texture(data))
    };

    let make_offset = |Size { width, height, .. }: Size| {
        let screen_inv = const { 1. / SCREEN_FACTOR as f32 };
        Vec2::new(
            screen_inv / width.get() as f32,
            screen_inv / height.get() as f32,
        )
    };

    let render_buffer = make_render_buffer((1, 1));
    let mut map = Map {
        tex: render_buffer.borrow().bind(),
        sam: cx.make_sampler(Filter::Nearest),
        offset: cx.make_uniform(&make_offset(render_buffer.borrow().size())),
    };

    let map_set = RefCell::new(cx.make_set(&screen_shader, &map));
    let handler = map_set.borrow().handler(&screen_shader);

    let screen_mesh = {
        const VERTS: [[Screen; 4]; 1] = [[
            Screen(Vec2::new(-1., -1.), Vec2::new(0., 1.)),
            Screen(Vec2::new(1., -1.), Vec2::new(1., 1.)),
            Screen(Vec2::new(1., 1.), Vec2::new(1., 0.)),
            Screen(Vec2::new(-1., 1.), Vec2::new(0., 0.)),
        ]];

        let data = MeshData::from_quads(&VERTS)?;
        cx.make_mesh(&data)
    };

    let window = {
        let attr = Attributes::default()
            .with_title("ssaa")
            .with_canvas(Canvas::by_id("root"));

        control.make_window(&cx, attr).await?
    };

    let triangle_layer = cx.make_layer(&shader, render_buffer.borrow().format());
    let screen_layer = cx.make_layer(&screen_shader, window.format());

    let bg = window.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;
            update_scene(redraw.delta_time());

            cx.shed(|mut s| {
                // draw the frame to the render buffer
                s.render(&*render_buffer.borrow(), bg)
                    .layer(&triangle_layer)
                    .set(&set)
                    .draw_points(3);

                // draw from the render buffer to the window
                s.render(&redraw, Options::default())
                    .layer(&screen_layer)
                    .set(&*map_set.borrow())
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
