use dunge::{
    color::Standard,
    handles::*,
    input::{Input, Key},
    transform::Position,
    CanvasConfig, Context, Error, Frame, InitialState, Loop, MeshData, Vertex, WindowMode,
};

#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3]);

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
    layer: LayerHandle<Vert>,
    mesh: MeshHandle<Vert>,
    instance: InstanceHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a shader and layer
        let layer = {
            use dunge_shader::{Camera, Dimension, Fragment, Scheme, Vertex};

            let shader = context.create_shader(Scheme {
                vert: Vertex {
                    dimension: Dimension::D2,
                    fragment: Fragment {
                        vertex_color: true,
                        vertex_texture: false,
                    },
                },
                base_color: None,
                camera: Camera::None,
            });

            context
                .create_layer_with_parameters()
                .build(shader)
                .expect("create layer")
        };

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
        let instance = {
            let data = Position::default();
            context.create_instances([data])
        };

        Self {
            layer,
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
        let mut layer = frame
            .layer(self.layer)?
            .with_clear_color(Standard([0, 0, 0, u8::MAX]))
            .with_clear_depth()
            .start();

        layer.bind_instance(self.instance)?;
        layer.draw(self.mesh)
    }
}
