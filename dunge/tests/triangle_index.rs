#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            prelude::*,
            sl::{self, Index, Render},
            Format,
        },
        glam::Vec4,
        helpers::image::Image,
        std::{env, f32::consts, fs},
    };

    let triangle = |Index(index): Index| {
        let color = const { Vec4::new(1., 0., 0., 1.) };
        let third = const { consts::TAU / 3. };
        let r_offset = const { -consts::TAU / 4. };
        let y_offset = 0.25;

        let i = sl::thunk(sl::f32(index) * third + r_offset);
        Render {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i) + y_offset, 0., 1.),
            color,
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_index.wgsl"));

    let size = const { (300, 300) };
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let data = TextureData::empty(size, Format::SrgbAlpha)?
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let buffer = cx.make_copy_buffer(size);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame.layer(&layer, opts).bind_empty().draw_points(3);
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

    if env::var("DUNGE_TEST_OUTPUT").is_ok() {
        fs::write("tests/triangle_index_actual.png", image.encode())?;
    }

    Ok(())
}
