#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            prelude::*,
            sl::{self, InVertex, Out},
            Format,
        },
        glam::Vec2,
        helpers::image::Image,
        std::{env, fs},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert([f32; 2], [f32; 3]);

    let triangle = |vert: InVertex<Vert>| Out {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: sl::vec4_with(sl::fragment(vert.1), 1.),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_vertex.wgsl"));

    let size = const { (300, 300) };
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let data = TextureData::empty(size, Format::SrgbAlpha)?
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let mesh = {
        let data = const {
            MeshData::from_verts(&[
                Vert([0., -0.75], [1., 0., 0.]),
                Vert([0.866, 0.75], [0., 1., 0.]),
                Vert([-0.866, 0.75], [0., 0., 1.]),
            ])
        };

        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(size);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame.layer(&layer, opts).bind_empty().draw(&mesh);
        frame.copy_texture(&buffer, &view);
    });

    cx.draw_to(&view, draw);
    let mapped = helpers::block_on({
        let (tx, rx) = helpers::oneshot();
        cx.map_view(buffer.view(), tx, rx)
    });

    let data = mapped.data();
    let image = Image::from_fn(size, |x, y| {
        let (width, _) = buffer.size();
        let idx = x + y * width;
        data[idx as usize]
    });

    if env::var("DUNGE_TEST_OUTPUT").is_ok() {
        fs::write("tests/triangle_vertex_actual.png", image.encode())?;
    }

    Ok(())
}
