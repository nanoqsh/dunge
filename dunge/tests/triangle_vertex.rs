#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        color::Rgba,
        draw,
        format::Format,
        mesh,
        sl::{self, InVertex, Out},
        texture, Vertex,
    },
    glam::Vec2,
    helpers::Image,
    std::{error, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    const SIZE: (u32, u32) = (300, 300);

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert([f32; 2], [f32; 3]);

    let triangle = |vert: InVertex<Vert>| Out {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: sl::vec4_with(sl::fragment(vert.1), 1.),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    fs::write("tests/triangle_vertex.wgsl", shader.debug_wgsl())?;

    let layer = cx.make_layer(&shader, Format::RgbAlpha);
    let view = {
        use texture::Data;

        let data = Data::empty(SIZE, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let mesh = {
        use mesh::Data;

        const VERTS: [Vert; 3] = [
            Vert([0., -0.75], [1., 0., 0.]),
            Vert([0.866, 0.75], [0., 1., 0.]),
            Vert([-0.866, 0.75], [0., 0., 1.]),
        ];

        let data = Data::from_verts(&VERTS);
        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, opts).bind_empty().draw(&mesh);
        frame.copy_texture(&buffer, &view);
    });

    cx.draw_to(&view, draw);
    let mapped = helpers::block_on({
        let (tx, rx) = helpers::oneshot();
        cx.map_view(buffer.view(), tx, rx)
    });

    let data = mapped.data();
    let image = Image::from_fn(SIZE, |x, y| {
        let (width, _) = buffer.size();
        let idx = x + y * width;
        data[idx as usize]
    });

    fs::write("tests/triangle_vertex.png", image.encode())?;
    Ok(())
}
