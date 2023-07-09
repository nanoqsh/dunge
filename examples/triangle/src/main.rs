use dunge::{
    error::NotSet,
    input::{Input, Key},
    shader::Shader,
    CanvasConfig, Context, Frame, InitialState, Instance, Layer, Loop, Mesh, MeshData, Rgba,
    Vertex, WindowMode,
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
            width: 500,
            height: 500,
        },
        ..Default::default()
    })
    .run_blocking(CanvasConfig::default(), App::new)
    .into_panic();
}

struct App {
    layer: Layer<TriangleShader>,
    mesh: Mesh<Vert>,
    instance: Instance,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a layer
        let layer = context.create_layer();

        // Create a mesh
        let mesh = {
            const VERTICES: [Vert; 3] = [
                Vert([-0.5, -0.5], [1., 0., 0.]),
                Vert([0.5, -0.5], [0., 1., 0.]),
                Vert([0., 0.5], [0., 0., 1.]),
            ];

            let data = MeshData::from_verts(&VERTICES);
            context.create_mesh(&data)
        };

        // Create a model instance
        let instance = context.create_default_instance();
        Self {
            layer,
            mesh,
            instance,
        }
    }
}

impl Loop for App {
    type Error = NotSet;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        // Handle pressed keys
        for key in input.pressed_keys {
            if key == Key::Escape {
                context.plan_to_close();
                return Ok(());
            }
        }

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        frame
            .layer(&self.layer)
            .with_clear_color(Rgba::from_bytes([0, 0, 0, u8::MAX]))
            .with_clear_depth()
            .start()
            .draw(&self.mesh, &self.instance)
    }
}
