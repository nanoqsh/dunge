use dunge::{
    input::Key, CanvasConfig, Context, Frame, InitialState, Input, Layer, Loop, Mesh, MeshData,
    Rgba, Shader, Vertex, WindowMode,
};

#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3]);

struct TriangleShader;

impl Shader for TriangleShader {
    type Vertex = Vert;
}

fn main() {
    env_logger::init();
    dunge::make_window(InitialState {
        title: "Triangle",
        mode: WindowMode::Windowed {
            width: 400,
            height: 400,
        },
        ..Default::default()
    })
    .run_blocking(CanvasConfig::default(), App::new)
    .expect("loop error");
}

struct App {
    layer: Layer<TriangleShader>,
    mesh: Mesh<Vert>,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a layer
        let layer = context.create_layer();

        // Create a mesh
        let mesh = {
            let data = MeshData::from_verts(&[
                Vert([-0.5, -0.5], [1., 0., 0.]),
                Vert([0.5, -0.5], [0., 1., 0.]),
                Vert([0., 0.5], [0., 0., 1.]),
            ]);

            context.create_mesh(&data)
        };

        Self { layer, mesh }
    }
}

impl Loop for App {
    fn update(&mut self, context: &mut Context, input: &Input) {
        // Handle pressed keys
        for key in input.pressed_keys {
            if key == Key::Escape {
                context.plan_to_close();
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame
            .layer(&self.layer)
            .with_clear_color(Rgba::from_bytes([0, 0, 0, u8::MAX]))
            .start()
            .draw(&self.mesh);
    }
}
