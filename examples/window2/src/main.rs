use dunge_winit::runtime::Control;

type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(e) = dunge_winit::runtime::block_on(run) {
        eprintln!("error: {e}");
    }
}

async fn run(ctrl: Control<'_>) -> Result<(), Error> {
    use {
        dunge_winit::{prelude::*, runtime::Attributes},
        futures_lite::future,
    };

    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: [f32; 2],
        col: [f32; 3],
    }

    let triangle = |vert: sl::InVertex<Vert>| {
        let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));
        let fragment_col = sl::fragment(vert.col);
        let color = sl::vec4_with(fragment_col, 1.);
        sl::Render { place, color }
    };

    let cx = ctrl.context();
    let _shader = cx.make_shader(triangle);

    let _mesh = {
        let verts = const {
            MeshData::from_verts(&[
                Vert {
                    pos: [-0.5, -0.5],
                    col: [1., 0., 0.],
                },
                Vert {
                    pos: [0.5, -0.5],
                    col: [0., 1., 0.],
                },
                Vert {
                    pos: [0., 0.5],
                    col: [0., 0., 1.],
                },
            ])
        };

        cx.make_mesh(&verts)
    };

    let window = ctrl.make_window(Attributes::default()).await?;

    loop {
        let resize = async {
            let (width, height) = window.resized().await;
            println!("resized: {width} {height}");
            false
        };

        let close = async {
            window.close_requested().await;
            true
        };

        if future::or(resize, close).await {
            break;
        }
    }

    Ok(())
}
