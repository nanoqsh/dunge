#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        color::Rgba,
        draw,
        format::Format,
        group::BoundTexture,
        mesh,
        sl::{self, Groups, InVertex, Out},
        state::Render,
        texture::{self, Filter, Sampler},
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
        pos: [f32; 2],
        tex: [f32; 2],
    }

    #[derive(Group)]
    struct Map<'a> {
        tex: BoundTexture<'a>,
        sam: &'a Sampler,
    }

    let triangle = |vert: InVertex<Vert>, Groups(map): Groups<Map>| Out {
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

    let layer = cx.make_layer(&shader, Format::RgbAlpha);
    let view = {
        use texture::Data;

        let data = Data::empty(SIZE, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let mesh = {
        use mesh::Data;

        const VERTS: [Vert; 3] = [
            Vert {
                pos: [0., -0.75],
                tex: [0., 1.],
            },
            Vert {
                pos: [0.866, 0.75],
                tex: [1., 1.],
            },
            Vert {
                pos: [-0.866, 0.75],
                tex: [1., 0.],
            },
        ];

        let data = Data::from_verts(&VERTS);
        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, opts).bind(&map).draw(&mesh);
        frame.copy_texture(&buffer, &view);
    });

    Render::default().draw_to(&cx, &view, draw);
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
