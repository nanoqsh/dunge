#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            buffer::Size,
            color::{Format, Rgb},
            instance_old::{RowOld, RowSliceOld},
            prelude::*,
            sl_old::{self, Index, PassInstance, Render},
        },
        glam::{Vec2, Vec3},
        helpers::image::Image,
        std::{env, f32::consts, fs},
    };

    #[derive(Instance)]
    struct Transform<'slice>(RowOld<Vec2>, RowSliceOld<'slice, Vec3>);

    let triangle = |PassInstance(t): PassInstance<Transform<'_>>, Index(index): Index| {
        let triangle_size = 0.4;
        let third = const { consts::TAU / 3. };
        let r_offset = const { -consts::TAU / 4. };

        let i = sl_old::thunk(sl_old::f32(index) * third + r_offset);
        let p = sl_old::vec2(sl_old::cos(i.clone()), sl_old::sin(i)) * triangle_size + t.0;
        Render {
            place: sl_old::vec4_concat(p, Vec2::new(0., 1.)),
            color: sl_old::vec4_append(sl_old::fragment(t.1), 1.),
        }
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader_old(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_instance.wgsl"));

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let size = Size::try_from(size)?;
        let data = TextureData::empty(size, Format::SrgbAlpha)
            .render()
            .copy_from();

        cx.make_texture(data)
    };

    let cols;
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

        cols = cx.make_row(&COLS);

        Transform(cx.make_row(&POSS), cols.slice(..))
    };

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let bg = Rgb::from_bytes([0; 3]);
        cx.shed(|s| {
            s.render(&view, bg)
                .layer(&layer)
                .instance(&transform)
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
        fs::write("tests/triangle_instance_actual.png", image.encode())?;
    }

    Ok(())
}
