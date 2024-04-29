#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            instance::Row,
            prelude::*,
            sl::{self, InInstance, Index, Out},
            Format,
        },
        glam::Vec2,
        helpers::image::Image,
        std::{f32::consts, fs},
    };

    const SIZE: (u32, u32) = (300, 300);
    const TRIANGLE_SIZE: f32 = 0.4;
    const THIRD: f32 = consts::TAU / 3.;
    const R_OFFSET: f32 = -consts::TAU / 4.;

    #[derive(Instance)]
    struct Transform(Row<[f32; 2]>, Row<[f32; 3]>);

    let triangle = |t: InInstance<Transform>, Index(index): Index| {
        let i = sl::thunk(sl::f32(index) * THIRD + R_OFFSET);
        let p = sl::vec2(sl::cos(i.clone()), sl::sin(i)) * TRIANGLE_SIZE + t.0;
        Out {
            place: sl::vec4_concat(p, Vec2::new(0., 1.)),
            color: sl::vec4_with(sl::fragment(t.1), 1.),
        }
    };

    let cx = helpers::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_instance.wgsl"));

    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let data = TextureData::empty(SIZE, Format::SrgbAlpha)?
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let transform = {
        const POS: [[f32; 2]; 3] = [[0.0, -0.375], [0.433, 0.375], [-0.433, 0.375]];
        const COL: [[f32; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];

        Transform(cx.make_row(&POS), cx.make_row(&COL))
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame
            .layer(&layer, opts)
            .bind_empty()
            .instance(&transform)
            .draw_points(3);

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

    fs::write("tests/triangle_instance.png", image.encode())?;
    Ok(())
}
