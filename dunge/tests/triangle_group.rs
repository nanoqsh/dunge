#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        buffer::{Filter, Sampler, Size, Texture},
        color::{Format, Rgb},
        prelude::*,
    },
    glam::{Vec2, Vec4},
    helpers::image::Image,
    std::env,
};

#[derive(Clone, Copy, Value, Bytes)]
struct Vert {
    pos: Vec2,
    uv: Vec2,
}

#[derive(Input)]
struct Map {
    texture: Texture,
    sampler: Sampler,
}

#[derive(Clone, Copy, Value)]
struct Io {
    #[position]
    pos: Vec4,
    uv: Vec2,
}

#[dunge(vertex)]
fn vs(v: Vert) -> Io {
    Io {
        pos: sl::concat(v.pos, Vec2::new(0., 1.)),
        uv: v.uv,
    }
}

#[dunge(fragment)]
fn fs(io: Io, m: Map) -> Vec4 {
    sl::texture_sample(m.texture, m.sampler, io.uv)
}

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    let cx = dunge::block_on(dunge::context())?;
    let triangle = cx.make_shader(
        render! {
            vertex: Vert,
            groups: [Map],
            shaders: [vs, fs],
        }
        .inspect(|r| {
            helpers::eq_lines(r.debug().to_string(), include_str!("triangle_group.wgsl"));
        })?,
    );

    let map = {
        let texture = {
            let gradient = Image::decode(include_bytes!("gradient.png"));
            let size = Size::try_from(gradient.size)?;
            let data = TextureData::new(size, Format::SrgbAlpha, &gradient.data)?.bind();
            cx.make_texture(data)
        };

        let map = Map {
            texture: texture.texture(),
            sampler: cx.make_sampler(Filter::Nearest).sampler(),
        };

        cx.make_set(&triangle, map)
    };

    let size = (300, 300);
    let layer = cx.make_layer(&triangle, Format::SrgbAlpha);
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
                uv: Vec2::new(0., 1.),
            },
            Vert {
                pos: Vec2::new(0.866, 0.75),
                uv: Vec2::new(1., 1.),
            },
            Vert {
                pos: Vec2::new(-0.866, 0.75),
                uv: Vec2::new(1., 0.),
            },
        ];

        let data = MeshData::from_verts(&VERTS).expect("mesh data");
        cx.make_mesh(&data)
    };

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let bg = Rgb::from_bytes([0; 3]);
        cx.shed(|s| {
            s.render(&view, bg).layer(&layer).set(map).draw(&mesh);
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
        std::fs::write("tests/triangle_group_actual.png", image.encode())?;
    }

    Ok(())
}
