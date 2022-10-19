use dunge::{
    color::Srgba,
    input::{Input, Key},
    transform::Position,
    ColorVertex, Context, Error, Frame, InitialState, InstanceHandle, Loop, MeshData, MeshHandle,
    Perspective, View, ViewHandle, WindowMode,
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
    instance: InstanceHandle,
    mesh: MeshHandle,
    view: ViewHandle,
}

impl App {
    fn new(context: &mut Context) -> Self {
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

            const INDICES: [u16; 3] = [0, 1, 2];

            let data = MeshData::new(&VERTICES, &[INDICES]).expect("create mesh");
            context.create_mesh(data)
        };

        // Fill black before draw a frame
        let color = Srgba([0, 0, 0, 255]);
        context.set_clear_color(color);

        // Create the view
        let view = context.create_view::<Perspective>(View::default());

        Self {
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
        let mut layer = frame.start_color_layer(Srgba([0, 0, 0, 255]));

        layer.bind_view(self.view)?;
        layer.bind_instance(self.instance)?;
        layer.draw(self.mesh)?;

        Ok(())
    }
}
