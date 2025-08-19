#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            buffer::{Filter, Format, Sampler, Size},
            color::Rgb,
            glam::Vec2,
            group::BoundTexture,
            prelude::*,
            sl::{self, Groups, PassVertex, Render},
        },
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
    struct Map {
        tex: BoundTexture,
        sam: Sampler,
    }

    let triangle = |PassVertex(v): PassVertex<Vert>, Groups(map): Groups<Map>| {
        let place = sl::vec4_concat(v.pos, Vec2::new(0., 1.));
        let color = {
            let samp = sl::thunk(sl::texture_sample(map.tex, map.sam, sl::fragment(v.tex)));
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
            let size = Size::try_from(alpha.size)?;
            let data = TextureData::new(size, Format::SrgbAlpha, &alpha.data)?.bind();
            cx.make_texture(data)
        };

        let sampler = cx.make_sampler(Filter::Nearest);

        let map = Map {
            tex: texture.bind(),
            sam: sampler,
        };

        cx.make_set(&shader, map)
    };

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let size = Size::try_from(size)?;
        let data = TextureData::empty(size, Format::SrgbAlpha)
            .render()
            .copy_from();

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

        cx.make_mesh(&MeshData::from_verts(&VERTS).expect("mesh data"))
    };

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let bg = Rgb::from_bytes([!0, 0, 0]);
        cx.shed(|s| {
            s.render(&view, bg).layer(&layer).set(&map).draw(&mesh);
            s.copy(&view, &buf);
        })
        .await;

        cx.read(&mut buf).await
    })?;

    let data = bytemuck::cast_slice(&read);
    let row = view.bytes_per_row_aligned() / view.format().bytes();
    let image = Image::from_fn(size, |x, y| {
        let idx = x + y * row;
        data[idx as usize]
    });

    if env::var("DUNGE_TEST_OUTPUT").is_ok() {
        fs::write("tests/triangle_discard_actual.png", image.encode())?;
    }

    Ok(())
}
