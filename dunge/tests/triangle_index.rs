#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        color::Rgba,
        draw,
        sl::{self, Index, Out},
        state::{Options, Render},
        texture::{self, Format},
    },
    glam::Vec4,
    helpers::Image,
    std::{error, f32::consts, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    const SIZE: (u32, u32) = (300, 300);
    const COLOR: Vec4 = Vec4::new(1., 0., 0., 1.);
    const THIRD: f32 = consts::TAU / 3.;
    const R_OFFSET: f32 = -consts::TAU / 4.;
    const Y_OFFSET: f32 = 0.25;

    let triangle = |Index(index): Index| {
        let [x, y] = sl::thunk(sl::f32(index) * THIRD + R_OFFSET);
        Out {
            place: sl::vec4(sl::cos(x), sl::sin(y) + Y_OFFSET, 0., 1.),
            color: COLOR,
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    let layer = cx.make_layer(Format::RgbAlpha, &shader);
    let view = {
        use texture::Data;

        let data = Data::empty(SIZE, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let options = Options::default().with_clear(Rgba::from_standard([0., 0., 0., 1.]));
    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, options).bind_empty().draw_triangles(1);
        frame.copy_texture(&buffer, &view);
    });

    let mut render = Render::default();
    cx.draw_to_texture(&mut render, &view, draw);

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
