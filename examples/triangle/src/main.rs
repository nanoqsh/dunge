use dunge::{
    color::Srgba,
    handles::*,
    input::{Input, Key},
    transform::Position,
    vertex::ColorVertex,
    Context, Error, Frame, InitialState, Loop, MeshData, Perspective, View, WindowMode,
};

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
    .run_blocking(App::new);
}

struct App {
    layer: LayerHandle<ColorVertex>,
    instance: InstanceHandle,
    mesh: MeshHandle<ColorVertex>,
    view: ViewHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a layer
        let layer = context.create_layer();

        // Create a model instance
        let instance = {
            let data = Position::default();
            context.create_instances([data])
        };

        // Create a mesh
        let mesh = {
            const VERTICES: [ColorVertex; 3] = [
                ColorVertex {
                    pos: [-0.5, -0.5, 0.],
                    col: [1., 0., 0.],
                },
                ColorVertex {
                    pos: [0.5, -0.5, 0.],
                    col: [0., 1., 0.],
                },
                ColorVertex {
                    pos: [0., 0.5, 0.],
                    col: [0., 0., 1.],
                },
            ];

            let data = MeshData::from_triangles(&VERTICES).expect("create mesh");
            context.create_mesh(&data)
        };

        // Create the view
        let view = context.create_view::<Perspective>(View::default());

        Self {
            layer,
            instance,
            mesh,
            view,
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
            .with_clear_color(Srgba([0, 0, 0, 255]))
            .with_clear_depth()
            .start();

        layer.bind_view(self.view)?;
        layer.bind_instance(self.instance)?;
        layer.draw(self.mesh)?;

        Ok(())
    }
}
