use {
    dunge::{
        color::Srgba,
        input::{Input, Key},
        Context, Error, Frame, InitialState, InstanceData, InstanceHandle, Loop, MeshData,
        MeshHandle, TextureData, TextureHandle, TextureVertex, WindowMode,
    },
    utils::Camera,
};

fn main() {
    env_logger::init();
    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 500,
            height: 500,
        },
        ..InitialState::default()
    })
    .run_blocking(App::new);
}

struct App {
    texture: TextureHandle,
    instance: InstanceHandle,
    mesh: MeshHandle,
    camera: Camera,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create a texture
        let texture = {
            let image = utils::read_png(include_bytes!("../../assets/grass.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context.create_texture(data)
        };

        // Create a model instance
        let data = InstanceData::default();
        let instance = context.create_instances([data]);

        // Create a mesh
        let data = MeshData::new(&VERTICES, &INDICES).expect("create mesh");
        let mesh = context.create_mesh(data);

        // Set the clear color
        let color = Srgba([29, 39, 34, 255]);
        context.set_clear_color(color);

        // Create the camera
        let camera = Camera::default();

        Self {
            texture,
            instance,
            mesh,
            camera,
        }
    }
}

impl Loop for App {
    type Error = Error;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        const SENSITIVITY: f32 = 0.01;

        // Handle pressed keys
        for key in input.pressed_keys {
            if let Key::Escape = key {
                context.plan_to_close();
                return Ok(());
            }
        }

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let view = self.camera.view();
        context.set_view(view);

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        frame.bind_texture(self.texture)?;
        frame.set_instance(self.instance)?;
        frame.draw_mesh(self.mesh)?;

        Ok(())
    }
}

const VERTICES: [TextureVertex; 22] = [
    TextureVertex {
        pos: [1.0, 1.0, -1.0],
        map: [1.0, 0.0],
    },
    TextureVertex {
        pos: [1.0, -1.0, 1.0],
        map: [0.5, 0.5],
    },
    TextureVertex {
        pos: [1.0, -1.0, -1.0],
        map: [1.0, 0.5],
    },
    TextureVertex {
        pos: [1.0, 1.0, 1.0],
        map: [1.0, 0.0],
    },
    TextureVertex {
        pos: [-1.0, -1.0, 1.0],
        map: [0.5, 0.5],
    },
    TextureVertex {
        pos: [1.0, -1.0, 1.0],
        map: [1.0, 0.5],
    },
    TextureVertex {
        pos: [-1.0, 1.0, 1.0],
        map: [1.0, 0.0],
    },
    TextureVertex {
        pos: [-1.0, -1.0, -1.0],
        map: [0.5, 0.5],
    },
    TextureVertex {
        pos: [-1.0, -1.0, 1.0],
        map: [1.0, 0.5],
    },
    TextureVertex {
        pos: [-1.0, 1.0, -1.0],
        map: [1.0, 0.0],
    },
    TextureVertex {
        pos: [1.0, -1.0, -1.0],
        map: [0.5, 0.5],
    },
    TextureVertex {
        pos: [-1.0, -1.0, -1.0],
        map: [1.0, 0.5],
    },
    TextureVertex {
        pos: [-1.0, -1.0, 1.0],
        map: [0.5, 1.0],
    },
    TextureVertex {
        pos: [1.0, -1.0, -1.0],
        map: [0.0, 0.5],
    },
    TextureVertex {
        pos: [1.0, -1.0, 1.0],
        map: [0.0, 1.0],
    },
    TextureVertex {
        pos: [1.0, 1.0, 1.0],
        map: [0.5, 0.5],
    },
    TextureVertex {
        pos: [-1.0, 1.0, -1.0],
        map: [0.0, 0.0],
    },
    TextureVertex {
        pos: [-1.0, 1.0, 1.0],
        map: [0.0, 0.5],
    },
    TextureVertex {
        pos: [1.0, 1.0, 1.0],
        map: [0.5, 0.0],
    },
    TextureVertex {
        pos: [-1.0, 1.0, 1.0],
        map: [0.5, 0.0],
    },
    TextureVertex {
        pos: [-1.0, 1.0, -1.0],
        map: [0.5, 0.0],
    },
    TextureVertex {
        pos: [1.0, 1.0, -1.0],
        map: [0.5, 0.0],
    },
];

const INDICES: [[u16; 3]; 12] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [9, 10, 11],
    [12, 13, 14],
    [15, 16, 17],
    [0, 18, 1],
    [3, 19, 4],
    [6, 20, 7],
    [9, 21, 10],
    [12, 7, 13],
    [15, 21, 16],
];