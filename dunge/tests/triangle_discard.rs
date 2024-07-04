#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            group::BoundTexture,
            prelude::*,
            sl::{self, Groups, InVertex, Out},
            texture::{Filter, Sampler},
            Format,
        },
        glam::Vec2,
        helpers::image::Image,
        std::fs,
    };

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

    let triangle = |vert: InVertex<Vert>, Groups(map): Groups<Map>| {
        let place = sl::vec4_concat(vert.pos, Vec2::new(0., 1.));
        let color = {
            let samp = sl::thunk(sl::texture_sample(map.tex, map.sam, sl::fragment(vert.tex)));
            let alpha = samp.clone().z();
            sl::if_then_else(sl::lt(alpha, 0.5), sl::discard, || samp)
        };

        Out { place, color }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_discard.wgsl"));

    let map = {
        let texture = {
            let alpha = Image::decode(include_bytes!("alpha.png"));
            let data = TextureData::new(&alpha.data, alpha.size, Format::SrgbAlpha)?.with_bind();
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
            ])
        };

        cx.make_mesh(&data)
    };

    let buffer = cx.make_copy_buffer(size);
    let opts = Rgba::from_standard([1., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame.layer(&layer, opts).bind(&map).draw(&mesh);
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

    fs::write("tests/triangle_discard.png", image.encode())?;
    Ok(())
}
