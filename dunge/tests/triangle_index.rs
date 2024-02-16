#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            sl::{self, Index, Out},
            texture::TextureData,
            Format,
        },
        glam::Vec4,
        helpers::image::Image,
        std::{f32::consts, fs},
    };

    const SIZE: (u32, u32) = (300, 300);
    const COLOR: Vec4 = Vec4::new(1., 0., 0., 1.);
    const THIRD: f32 = consts::TAU / 3.;
    const R_OFFSET: f32 = -consts::TAU / 4.;
    const Y_OFFSET: f32 = 0.25;

    let triangle = |Index(index): Index| {
        let i = sl::thunk(sl::f32(index) * THIRD + R_OFFSET);
        Out {
            place: sl::vec4(sl::cos(i.clone()), sl::sin(i) + Y_OFFSET, 0., 1.),
            color: COLOR,
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_index.wgsl"));

    let layer = cx.make_layer(&shader, Format::RgbAlpha);
    let view = {
        let data = TextureData::empty(SIZE, Format::RgbAlpha)?
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
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
    let image = Image::from_fn(SIZE, |x, y| {
        let (width, _) = buffer.size();
        let idx = x + y * width;
        data[idx as usize]
    });

    fs::write("tests/triangle_index.png", image.encode())?;
    Ok(())
}
