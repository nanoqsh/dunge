#![cfg(not(target_family = "wasm"))]

use {
    dunge::{
        color::Rgba,
        draw,
        format::Format,
        instance::{Projection, Row, Set, SetMember, Setter},
        sl::{self, Define, InInstance, Index, Out, ReadInstance, Ret},
        state::Render,
        texture,
        types::{self, VectorType},
        Instance,
    },
    glam::Vec2,
    helpers::Image,
    std::{error, f32::consts, fs},
};

type Error = Box<dyn error::Error>;

#[test]
fn render() -> Result<(), Error> {
    const SIZE: (u32, u32) = (300, 300);
    const TRIANGLE_SIZE: f32 = 0.4;
    const THIRD: f32 = consts::TAU / 3.;
    const R_OFFSET: f32 = -consts::TAU / 4.;

    struct Transform {
        pos: Row<[f32; 2]>,
        col: Row<[f32; 3]>,
    }

    impl Instance for Transform {
        type Projection = TransformProjection;
        const DEF: Define<VectorType> = Define::new(&[VectorType::Vec2f, VectorType::Vec3f]);
    }

    impl Set for Transform {
        fn set<'p>(&'p self, setter: &mut Setter<'_, 'p>) {
            <Row<[f32; 2]> as SetMember>::set_member(&self.pos, setter);
            <Row<[f32; 3]> as SetMember>::set_member(&self.col, setter);
        }
    }

    struct TransformProjection {
        pos: Ret<ReadInstance, types::Vec2<f32>>,
        col: Ret<ReadInstance, types::Vec3<f32>>,
    }

    impl Projection for TransformProjection {
        fn projection(id: u32) -> Self {
            Self {
                pos: ReadInstance::new(id),
                col: ReadInstance::new(id + 1),
            }
        }
    }

    let triangle = |t: InInstance<Transform>, Index(index): Index| {
        let [x, y] = sl::thunk(sl::f32(index) * THIRD + R_OFFSET);
        let p = sl::vec2(sl::cos(x), sl::sin(y)) * TRIANGLE_SIZE + t.pos;
        Out {
            place: sl::concat(p, Vec2::new(0., 1.)),
            color: sl::vec4_with(sl::fragment(t.col), 1.),
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

    let transform = {
        const POS: [[f32; 2]; 3] = [[0.0, -0.375], [0.433, 0.375], [-0.433, 0.375]];
        const COL: [[f32; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];

        Transform {
            pos: cx.make_row(&POS),
            col: cx.make_row(&COL),
        }
    };

    let buffer = cx.make_copy_buffer(SIZE);
    let opts = Rgba::from_standard([0., 0., 0., 1.]);
    let draw = draw::from_fn(|mut frame| {
        frame
            .layer(&layer, opts)
            .bind_empty()
            .instance(&transform)
            .draw_triangles(1);

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

    fs::write("tests/triangle_instance.png", image.encode())?;
    Ok(())
}
