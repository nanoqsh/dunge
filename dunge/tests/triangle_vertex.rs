use {
    dunge::{
        color::Rgba,
        context::Context,
        draw, mesh,
        sl::{self, Input, Out},
        state::{Options, Render},
        texture::{self, Format},
        Vertex,
    },
    glam::{Vec2, Vec3},
    helpers::Image,
    std::{error, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    const SIZE: (u32, u32) = (300, 300);

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec2,
        col: Vec3,
    }

    let triangle = |vert: Input<Vert>| Out {
        place: sl::concat(vert.pos, Vec2::new(0., 1.)),
        color: sl::vec4_with(sl::fragment(vert.col), 1.),
    };

    let cx = helpers::block_on(Context::new())?;
    let shader = cx.make_shader(triangle);
    let layer = cx.make_layer(Format::RgbAlpha, &shader);
    let view = {
        use texture::Data;

        let data = Data::empty(SIZE, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let mesh = {
        use mesh::Data;

        const VERTS: [Vert; 3] = [
            Vert {
                pos: Vec2::new(0., -0.75),
                col: Vec3::new(1., 0., 0.),
            },
            Vert {
                pos: Vec2::new(0.866, 0.75),
                col: Vec3::new(0., 1., 0.),
            },
            Vert {
                pos: Vec2::new(-0.866, 0.75),
                col: Vec3::new(0., 0., 1.),
            },
        ];

        let data = Data::from_verts(&VERTS);
        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let options = Options::default().with_clear(Rgba::from_standard([0., 0., 0., 1.]));
    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, options).bind_empty().draw(&mesh);
        frame.copy_texture(&buffer, &view);
    });

    let mut render = Render::default();
    cx.draw_to_texture(&mut render, &view, draw);

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
