use {
    dunge::{
        color::Rgba,
        context::Context,
        draw,
        sl::{self, Index, Out},
        state::{Options, Render},
        texture::{Data, Format},
    },
    glam::Vec4,
    helpers::Image,
    std::{error, f32::consts, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    const SIZE: (u32, u32) = (300, 300);
    const THIRD: f32 = consts::TAU / 3.;

    let triangle = |Index(index): Index| {
        let [x, y] = sl::share(sl::f32(index) * THIRD);
        Out {
            place: sl::vec4(sl::cos(x), sl::sin(y), 0., 1.),
            color: Vec4::new(1., 0., 0., 1.),
        }
    };

    let cx = helpers::block_on(Context::new())?;
    let shader = cx.make_shader(triangle);
    let bind = cx.make_binder(&shader).into_binding();
    let layer = cx.make_layer(Format::RgbAlpha, &shader);

    let view = {
        let data = Data::empty(SIZE, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let clear = Rgba::from_standard([0., 0., 0., 1.]);
    let buffer = cx.make_copy_buffer(SIZE);
    let options = Options {
        clear_color: Some(clear),
    };

    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, options).bind(&bind).draw_triangles(1);
        frame.copy_texture(&buffer, &view);
    });

    let mut render = Render::default();
    cx.draw_to_texture(&mut render, &view, draw);

    let mapped = helpers::block_on({
        let (tx, rx) = helpers::oneshot();
        cx.map_view(buffer.view(), tx, rx)
    });

    let mapped = mapped.data();
    let image = Image::from_fn(SIZE, |x, y| {
        let (width, _) = buffer.size();
        let idx = x + y * width;
        mapped[idx as usize]
    });

    let data = image.encode();
    fs::write("tests/triangle.png", data)?;
    Ok(())
}
