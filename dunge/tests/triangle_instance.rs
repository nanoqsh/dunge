#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        buffer::Size,
        color::{Format, Rgb},
        prelude::*,
    },
    glam::{Vec2, Vec3, Vec4},
    helpers::image::Image,
    std::{env, f32::consts},
};

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Copy, Value)]
struct Transform {
    pos: Vec2,
    col: Vec3,
}

#[derive(Clone, Copy, Value)]
struct Index {
    #[index]
    index: u32,
}

#[derive(Clone, Copy, Value)]
struct Io {
    #[position]
    pos: Vec4,
    col: Vec3,
}

#[dunge(vertex)]
fn vs(t: Transform, ind: Index) -> Io {
    let size = const { 0.4 };
    let third = const { consts::TAU / 3. };
    let r_offset = const { -consts::TAU / 4. };
    let i = ind.index as f32 * third + r_offset;
    let p = Vec2::new(sl::cos(i), sl::sin(i)) * size + t.pos;
    Io {
        pos: sl::concat(p, Vec2::new(0., 1.)),
        col: t.col,
    }
}

#[dunge(fragment)]
fn fs(io: Io) -> Vec4 {
    sl::append(io.col, 1.)
}

#[test]
fn render() -> Result<(), Error> {
    let cx = dunge::block_on(dunge::context())?;
    let triangle = cx.make_shader(
        render! {
            instance: Transform,
            shaders: [vs, fs],
        }
        .inspect(|r| {
            helpers::eq_lines(
                r.debug().to_string(),
                include_str!("triangle_instance.wgsl"),
            );
        })?,
    );

    let size = (300, 300);
    let layer = cx.make_layer(&triangle, Format::SrgbAlpha);
    let view = {
        let size = Size::try_from(size)?;
        let data = TextureData::empty(size, Format::SrgbAlpha)
            .render()
            .copy_from();

        cx.make_texture(data)
    };

    let (poss, cols) = {
        const POSS: [Vec2; 3] = [
            Vec2::new(0., -0.375),
            Vec2::new(0.433, 0.375),
            Vec2::new(-0.433, 0.375),
        ];

        const COLS: [Vec3; 3] = [
            Vec3::new(1., 0., 0.),
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 0., 1.),
        ];

        let poss = cx.make_row(&POSS).ok_or("failed to make an instance")?;
        let cols = cx.make_row(&COLS).ok_or("failed to make an instance")?;
        (poss, cols)
    };

    let cols = cols.slice(..).expect("row slice");

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let bg = Rgb::from_bytes([0; 3]);
        cx.shed(|s| {
            s.render(&view, bg)
                .layer(&layer)
                .instance((poss, cols))
                .draw_points(3);

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
        std::fs::write("tests/triangle_instance_actual.png", image.encode())?;
    }

    Ok(())
}
