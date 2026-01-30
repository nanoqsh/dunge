#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        buffer::Size,
        color::{Format, Rgb},
        prelude::*,
    },
    glam::Vec4,
    helpers::image::Image,
    std::{env, f32::consts},
};

#[derive(Clone, Copy, Value)]
struct Index {
    #[index]
    index: u32,
}

#[dunge(vertex)]
fn vs(ind: Index) -> Vec4 {
    let third = const { consts::TAU / 3. };
    let r_offset = const { -consts::TAU / 4. };
    let y_offset = const { 0.25 };
    let i = ind.index as f32 * third + r_offset;
    Vec4::new(sl::cos(i), sl::sin(i) + y_offset, 0., 1.)
}

#[dunge(fragment)]
fn fs() -> Vec4 {
    Vec4::new(1., 0., 0., 1.)
}

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    let cx = dunge::block_on(dunge::context())?;
    let triangle = cx.make_shader(
        render! {
            shaders: [vs, fs],
        }
        .inspect(|r| {
            helpers::eq_lines(r.debug().to_string(), include_str!("triangle_index.wgsl"));
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

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let bg = Rgb::from_bytes([0; 3]);
        cx.shed(|s| {
            s.render(&view, bg).layer(&layer).draw_points(3);
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
        std::fs::write("tests/triangle_index_actual.png", image.encode())?;
    }

    Ok(())
}
