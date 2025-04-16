#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            color::Rgba,
            instance::Row,
            prelude::*,
            sl::{self, InInstance, Index, Render},
            texture::Size,
        },
        glam::{Vec2, Vec3},
        helpers::image::Image,
        std::{env, f32::consts, fs},
    };

    #[derive(Instance)]
    struct Transform(Row<Vec2>, Row<Vec3>);

    let triangle = |t: InInstance<Transform>, Index(index): Index| {
        let triangle_size = 0.4;
        let third = const { consts::TAU / 3. };
        let r_offset = const { -consts::TAU / 4. };

        let i = sl::thunk(sl::f32(index) * third + r_offset);
        let p = sl::vec2(sl::cos(i.clone()), sl::sin(i)) * triangle_size + t.0;
        Render {
            place: sl::vec4_concat(p, Vec2::new(0., 1.)),
            color: sl::vec4_with(sl::fragment(t.1), 1.),
        }
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_instance.wgsl"));

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let size = Size::try_from(size)?;
        let data = TextureData::empty(size, Format::SrgbAlpha)
            .with_draw()
            .with_copy();

        cx.make_texture(data)
    };

    let transform = {
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

        Transform(cx.make_row(&POSS), cx.make_row(&COLS))
    };

    let buffer = cx.make_copy_buffer(size);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = dunge::draw(|mut frame| {
        frame
            .set_layer(&layer, opts)
            .bind_empty()
            .instance(&transform)
            .draw_points(3);

        frame.copy_texture(&buffer, &view);
    });

    cx.draw_to(&view, draw);
    let mapped = dunge::block_on({
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
        fs::write("tests/triangle_instance_actual.png", image.encode())?;
    }

    Ok(())
}
