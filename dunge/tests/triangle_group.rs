#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        color::Rgba,
        draw,
        group::BoundTexture,
        mesh,
        sl::{self, Groups, Input, Out},
        state::{Options, Render},
        texture::{self, Filter, Format, Sampler},
        Group, Vertex,
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
    struct Vert {
        pos: Vec2,
        tex: Vec2,
    }

    #[derive(Group)]
    struct Map<'a> {
        tex: BoundTexture<'a>,
        sam: &'a Sampler,
    }

    let triangle = |vert: Input<Vert>, Groups(map): Groups<Map>| Out {
        place: sl::concat(vert.pos, Vec2::new(0., 1.)),
        color: sl::texture_sample(map.tex, map.sam, sl::fragment(vert.tex)),
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    let map = {
        let texture = {
            use texture::Data;

            let gradient = Image::decode(include_bytes!("gradient.png"));
            let data = Data::new(&gradient.data, gradient.size, Format::RgbAlpha)?.with_bind();
            cx.make_texture(data)
        };

        let sampler = cx.make_sampler(Filter::Nearest);
        let map = Map {
            tex: BoundTexture::new(&texture),
            sam: &sampler,
        };

        let mut binder = cx.make_binder(&shader);
        binder.bind(&map);
        binder.into_binding()
    };

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
                tex: Vec2::new(0., 1.),
            },
            Vert {
                pos: Vec2::new(0.866, 0.75),
                tex: Vec2::new(1., 1.),
            },
            Vert {
                pos: Vec2::new(-0.866, 0.75),
                tex: Vec2::new(1., 0.),
            },
        ];

        let data = Data::from_verts(&VERTS);
        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let options = Options::default().with_clear(Rgba::from_standard([0., 0., 0., 1.]));
    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, options).bind(&map).draw(&mesh);
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

    fs::write("tests/triangle_group.png", image.encode())?;
    Ok(())
}