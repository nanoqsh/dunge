#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            group::BoundTexture,
            prelude::*,
            sl::{self, Groups, InVertex, Render},
            texture::{Filter, Sampler},
            Format,
        },
        glam::Vec2,
        helpers::image::Image,
        std::{env, fs},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: Vec2,
        tex: Vec2,
    }

    #[derive(Group)]
    struct Map<'tex> {
        tex: BoundTexture<'tex>,
        sam: &'tex Sampler,
    }

    let triangle = |vert: InVertex<Vert>, Groups(map): Groups<Map<'_>>| {
        let place = sl::vec4_concat(vert.pos, Vec2::new(0., 1.));
        let color = {
            let samp = sl::thunk(sl::texture_sample(map.tex, map.sam, sl::fragment(vert.tex)));
            let alpha = samp.clone().z();
            sl::if_then_else(sl::lt(alpha, 0.5), sl::discard, || samp)
        };

        Render { place, color }
    };

    let cx = dunge::block_on(dunge::context())?;
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

        cx.make_set(&shader, map)
    };

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let data = TextureData::empty(size, Format::SrgbAlpha)?
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let mesh = {
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

        const DATA: MeshData<'_, Vert> = MeshData::from_verts(&VERTS);

        cx.make_mesh(&DATA)
    };

    let buffer = cx.make_copy_buffer(size);
    let opts = Rgba::from_standard([1., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame.set_layer(&layer, opts).with(&map).draw(&mesh);
        frame.copy_texture(&buffer, &view);
    });

    cx.draw_to(&view, draw);
    let mapped = dunge::block_on({
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
        fs::write("tests/triangle_discard_actual.png", image.encode())?;
    }

    Ok(())
}
