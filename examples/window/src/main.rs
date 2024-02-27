type Error = Box<dyn std::error::Error>;

fn main() {
    env_logger::init();
    if let Err(err) = helpers::block_on(run()) {
        eprintln!("error: {err}");
    }
}

async fn run() -> Result<(), Error> {
    use dunge::prelude::*;

    // Create a vertex type
    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        pos: [f32; 2],
        col: [f32; 3],
    }

    // Create a shader program
    let triangle = |vert: sl::InVertex<Vert>| {
        // Describe the vertex position:
        // Take the vertex data as vec2 and expand it to vec4
        let place = sl::vec4_concat(vert.pos, sl::vec2(0., 1.));

        // Then describe the vertex color:
        // First you need to pass the color from
        // vertex shader stage to fragment shader stage
        let fragment_col = sl::fragment(vert.col);

        // Now create the final color by adding an alpha value
        let color = sl::vec4_with(fragment_col, 1.);

        // As a result, return a program that describes how to
        // compute the vertex position and the fragment color
        sl::Out { place, color }
    };

    // Create the dunge context with a window
    let window = dunge::window().await?;
    let cx = window.context();

    // You can use the context to manage dunge objects.
    // Create a shader instance
    let shader = cx.make_shader(triangle);
    // And a layer for drawing a mesh on it
    let layer = cx.make_layer(&shader, window.format());

    // Create a mesh from vertices
    let mesh = {
        use dunge::mesh::MeshData;

        const VERTS: MeshData<'static, Vert> = MeshData::from_verts(&[
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
        ]);

        cx.make_mesh(&VERTS)
    };

    // Describe the `Update` handler
    let upd = |ctrl: &Control| {
        for key in ctrl.pressed_keys() {
            // Exit by pressing escape key
            if key.code == KeyCode::Escape {
                return Then::Close;
            }
        }

        // Otherwise continue running
        Then::Run
    };

    // Describe the `Draw` handler
    let draw = |mut frame: Frame| {
        use dunge::color::Rgba;

        // Create a black RGBA background
        let bg = Rgba::from_bytes([0, 0, 0, !0]);

        frame
            // Select a layer to draw on it
            .layer(&layer, bg)
            // The shader has no bindings, so call empty bind
            .bind_empty()
            // And finally draw the mesh
            .draw(&mesh);
    };

    // Run the window with handlers
    window.run(dunge::update(upd, draw))?;
    Ok(())
}
