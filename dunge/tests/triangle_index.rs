#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            buffer::Size,
            color::Rgb,
            prelude::*,
            sl::{self, Index, Render},
        },
        glam::Vec4,
        helpers::image::Image,
        std::{env, f32::consts, fs},
    };

    let triangle = |Index(index): Index| {
        let color = Vec4::new(1., 0., 0., 1.);
        let third = const { consts::TAU / 3. };
        let r_offset = const { -consts::TAU / 4. };
        let y_offset = 0.25;

        let i = sl::thunk(sl::f32(index) * third + r_offset);
        Render {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i) + y_offset, 0., 1.),
            color,
        }
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_index.wgsl"));

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
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
        cx.shed(|mut s| {
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
        fs::write("tests/triangle_index_actual.png", image.encode())?;
    }

    Ok(())
}
