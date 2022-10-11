use {
    dunge::{
        color::Srgba,
        input::{Input, Key},
        rotation::Identity,
        Context, Error, Frame, FrameFilter, InitialState, InstanceData, InstanceHandle, Loop,
        MeshData, MeshHandle, Perspective, TextureData, TextureHandle, TextureVertex, View,
        WindowMode,
    },
    image::DynamicImage as Image,
};

fn main() {
    env_logger::init();
    dunge::make_window(InitialState {
        mode: WindowMode::Windowed {
            width: 300,
            height: 300,
        },
        ..InitialState::default()
    })
    .run(App::new);
}

struct App {
    texture: TextureHandle,
    instance: InstanceHandle,
    mesh: MeshHandle,
    camera: Camera,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Set pixel size and frame filter.
        context.set_frame_parameters(2, FrameFilter::Nearest);

        // Create a texture
        let texture = {
            let image = read_png(include_bytes!("grass.png")).to_rgba8();
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context.create_texture(data)
        };

        type Data = InstanceData<Identity>;

        // Create a model instances
        let positions = [
            [0., 0., 0.],
            [2., 0., 0.],
            [0., 0., 2.],
            [-2., 0., 0.],
            [0., 0., -2.],
        ];
        let instance = context.create_instances(positions.map(|pos| Data {
            pos,
            ..Default::default()
        }));

        // Create a mesh
        let data = MeshData::new(&VERTICES, &INDICES).expect("create mesh");
        let mesh = context.create_mesh(data);

        // Set the clear color
        let color = Srgba([29, 39, 34, 255]);
        context.set_clear_color(color);

        // Create the camera
        let camera = Camera {
            angle: 0.,
            pitch: 0.,
            distance: 3.,
        };

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

fn read_png(bytes: &[u8]) -> Image {
    use {
        image::{io::Reader, ImageFormat},
        std::io::Cursor,
    };

    Reader::with_format(Cursor::new(bytes), ImageFormat::Png)
        .decode()
        .expect("decode png")
}

struct Camera {
    angle: f32,
    pitch: f32,
    distance: f32,
}

impl Camera {
    fn update(&mut self, (x, y, z): (f32, f32, f32)) {
        use std::f32::consts::TAU;

        self.angle -= x % TAU;
        self.pitch = (self.pitch + z).clamp(-1., 1.);
        self.distance = (self.distance - y).clamp(3., 10.);
    }

    fn view(&self) -> View<Perspective> {
        let x = self.angle.sin() * self.pitch.cos() * self.distance;
        let y = self.pitch.sin() * self.distance;
        let z = self.angle.cos() * self.pitch.cos() * self.distance;

        View {
            eye: [x, y, z],
            ..Default::default()
        }
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
