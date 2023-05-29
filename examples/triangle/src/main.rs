use dunge::{
    color::Standard,
    handles::*,
    input::{Input, Key},
    transform::Position,
    CanvasConfig, Context, Error, Frame, InitialState, Loop, MeshData, Shader, Vertex, WindowMode,
};

#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3]);

struct ColorShader;
impl Shader for ColorShader {
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
    layer: LayerHandle<ColorShader>,
    globals: GlobalsHandle<ColorShader>,
    mesh: MeshHandle<Vert>,
    instance: InstanceHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create shader and layer
        let layer = {
            use dunge_shader::{Dimension, Fragment, Scheme, Vertex, View};

            let shader: ShaderHandle<ColorShader> = context.create_shader(Scheme {
                vert: Vertex {
                    dimension: Dimension::D2,
                    fragment: Fragment {
                        vertex_color: true,
                        vertex_texture: false,
                    },
                },
                view: View::Camera,
                static_color: None,
                dynamic_color: false,
            });

            context
                .create_layer_with_parameters()
                .build(shader)
                .expect("create layer")
        };

        // Create globals
        let globals = context
            .globals_builder()
            .with_view()
            .build(layer)
            .expect("create globals");

        // Create a mesh
        let mesh = {
            const VERTICES: [Vert; 3] = [
                Vert([-100., -100.], [1., 0., 0.]),
                Vert([100., -100.], [0., 1., 0.]),
                Vert([0., 100.], [0., 0., 1.]),
            ];

            let data = MeshData::from_verts(&VERTICES);
            context.create_mesh(&data)
        };

        // Create a model instance
        let instance = {
            let data = Position::default();
            context.create_instances([data])
        };

        Self {
            layer,
            globals,
            mesh,
            instance,
        }
    }
}

impl Loop for App {
    type Error = Error;

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
            .layer(self.layer)?
            .with_clear_color(Standard([0, 0, 0, u8::MAX]))
            .with_clear_depth()
            .start()
            .bind_globals(self.globals)?
            .bind_instance(self.instance)?
            .draw(self.mesh)
    }
}
