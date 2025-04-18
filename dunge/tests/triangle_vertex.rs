#![cfg(not(target_family = "wasm"))]

type Error = Box<dyn std::error::Error>;

#[test]
fn render() -> Result<(), Error> {
    use {
        dunge::{
            buffer::Size,
            color::Rgba,
            prelude::*,
            sl::{self, InVertex, Render},
        },
        glam::{Vec2, Vec3},
        helpers::image::Image,
        std::{env, fs},
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert(Vec2, Vec3);

    let triangle = |vert: InVertex<Vert>| Render {
        place: sl::vec4_concat(vert.0, Vec2::new(0., 1.)),
        color: sl::vec4_with(sl::fragment(vert.1), 1.),
    };

    let cx = dunge::block_on(dunge::context())?;
    let shader = cx.make_shader(triangle);
    helpers::eq_lines(shader.debug_wgsl(), include_str!("triangle_vertex.wgsl"));

    let size = (300, 300);
    let layer = cx.make_layer(&shader, Format::SrgbAlpha);
    let view = {
        let size = Size::try_from(size)?;
        let data = TextureData::empty(size, layer.format())
            .render()
            .copy_from();

        cx.make_texture(data)
    };

    let mesh = {
        const VERTS: [Vert; 3] = [
            Vert(Vec2::new(0., -0.75), Vec3::new(1., 0., 0.)),
            Vert(Vec2::new(0.866, 0.75), Vec3::new(0., 1., 0.)),
            Vert(Vec2::new(-0.866, 0.75), Vec3::new(0., 0., 1.)),
        ];

        const DATA: MeshData<'_, Vert> = MeshData::from_verts(&VERTS);

        cx.make_mesh(&DATA)
    };

    let mut buf = {
        let data = view.copy_buffer_data().read();
        cx.make_buffer(data)
    };

    let read = dunge::block_on(async {
        let opts = Rgba::from_bytes([0, 0, 0, !0]);
        cx.shed(|mut s| {
            s.render(&view, opts).layer(&layer).draw(&mesh);
            s.copy(&view, &buf);
        })
        .await;

        cx.read(&mut buf).await
    })?;

    let data = bytemuck::cast_slice(&read);
    let image = Image::from_fn(size, |x, y| {
        let row = view.bytes_per_row_aligned() / view.format().bytes();
        let idx = x + y * row;
        data[idx as usize]
    });

    if env::var("DUNGE_TEST_OUTPUT").is_ok() {
        fs::write("tests/triangle_vertex_actual.png", image.encode())?;
    }

    Ok(())
}
