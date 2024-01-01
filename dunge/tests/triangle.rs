use {
    dunge::{
        color::Rgba,
        context::Context,
        draw,
        sl::{self, Index, Out},
        state::{Options, Render},
        texture::{Data, Format},
    },
    futures::future,
    glam::Vec4,
    helpers::Image,
    std::{error, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use std::f32::consts;

    const THIRD: f32 = consts::TAU / 3.;

    let triangle = |Index(index): Index| {
        let [x, y] = sl::share(sl::f32(index) * THIRD);
        Out {
            place: sl::vec4(sl::cos(x), sl::sin(y), 0., 1.),
            color: Vec4::new(1., 0., 0., 1.),
        }
    };

    let cx = future::block_on(Context::new())?;
    let shader = cx.make_shader(triangle);
    let bind = cx.make_binder(&shader).into_binding();
    let layer = cx.make_layer(Format::RgbAlpha, &shader);

    let size = (300, 300);
    let view = {
        let data = Data::empty(size, Format::RgbAlpha)?.with_draw().with_copy();
        cx.make_texture(data)
    };

    let clear = Rgba::from_standard([0., 0., 0., 1.]);
    let buffer = cx.make_copy_buffer(size);
    let options = Options {
        clear_color: Some(clear),
    };

    let draw = draw::from_fn(|mut frame| {
        frame.layer(&layer, options).bind(&bind).draw_triangles(1);
        frame.copy_texture(&buffer, &view);
    });

    let mut render = Render::default();
    cx.draw_to_texture(&mut render, &view, draw);

    let mapped = future::block_on({
        let (tx, rx) = async_channel::bounded(1);
        cx.map_view(
            buffer.view(),
            move |r| tx.send_blocking(r).expect("send mapped result"),
            || async move { rx.recv().await.expect("recv mapped result") },
        )
    });

    let mapped = mapped.data();
    let image = Image {
        data: {
            let (width, height) = size;
            let mut data = vec![0; (width * height * 4) as usize].into_boxed_slice();
            for y in 0..height {
                for x in 0..width {
                    let (actual_width, _) = buffer.size();
                    let idx = x + y * actual_width;
                    let loc = (x + y * width) * 4;
                    data[loc as usize..loc as usize + 4].copy_from_slice(&mapped[idx as usize]);
                }
            }

            data
        },
        size,
    };

    let data = helpers::encode_png(&image);
    fs::write("tests/triangle.png", data)?;
    Ok(())
}
