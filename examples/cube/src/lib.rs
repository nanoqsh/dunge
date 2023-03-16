use {
    dunge::{
        color::Srgba,
        input::{Input, Key},
        transform::Position,
        vertex::{ColorVertex, TextureVertex},
        Context, Error, Frame, InstanceHandle, Loop, MeshData, MeshHandle, Perspective,
        TextureData, TextureHandle, ViewHandle,
    },
    utils::Camera,
};

enum State {
    Texture,
    Color,
}

pub struct App {
    texture: TextureHandle,
    instance: InstanceHandle,
    texture_mesh: MeshHandle<TextureVertex>,
    color_mesh: MeshHandle<ColorVertex>,
    view: ViewHandle,
    camera: Camera,
    state: State,
}

impl App {
    pub fn new(context: &mut Context) -> Self {
        // Create a texture
        let texture = {
            let image = utils::read_png(include_bytes!("grass.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context.create_texture(data)
        };

        // Create a model instance
        let instance = {
            let data = Position::default();
            context.create_instances([data])
        };

        // Create meshes
        let texture_mesh = {
            let verts = VERTICES.map(|(pos, map)| TextureVertex { pos, map });
            let data = MeshData::new(&verts, &INDICES).expect("create mesh");
            context.create_mesh(&data)
        };

        let color_mesh = {
            let verts = VERTICES.map(|(pos, [a, b])| ColorVertex {
                pos,
                col: [a, b, 1.],
            });
            let data = MeshData::new(&verts, &INDICES).expect("create mesh");
            context.create_mesh(&data)
        };

        // Create the view
        let camera = Camera::default();
        let view = context.create_view(camera.view(Perspective::default()));

        Self {
            texture,
            instance,
            texture_mesh,
            color_mesh,
            view,
            camera,
            state: State::Texture,
        }
    }
}

impl Loop for App {
    type Error = Error;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        const SENSITIVITY: f32 = 0.01;

        // Handle pressed keys
        for key in input.pressed_keys {
            match key {
                Key::Escape => {
                    context.plan_to_close();
                    return Ok(());
                }
                Key::Space => match self.state {
                    State::Texture => self.state = State::Color,
                    State::Color => self.state = State::Texture,
                },
                _ => {}
            }
        }

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let view = self.camera.view(Perspective::default());
        context.update_view(self.view, view)?;

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        let color = Srgba([46, 34, 47, 255]);

        match self.state {
            State::Texture => {
                let mut layer = frame
                    .texture_layer()
                    .with_clear_color(color)
                    .with_clear_depth()
                    .start();

                layer.bind_view(self.view)?;
                layer.bind_instance(self.instance)?;
                layer.bind_texture(self.texture)?;
                layer.draw(self.texture_mesh)?;
            }
            State::Color => {
                let mut layer = frame
                    .color_layer()
                    .with_clear_color(color)
                    .with_clear_depth()
                    .start();

                layer.bind_view(self.view)?;
                layer.bind_instance(self.instance)?;
                layer.draw(self.color_mesh)?;
            }
        }

        Ok(())
    }
}

const VERTICES: [([f32; 3], [f32; 2]); 22] = [
    ([1.0, 1.0, -1.0], [1.0, 0.0]),
    ([1.0, -1.0, 1.0], [0.5, 0.5]),
    ([1.0, -1.0, -1.0], [1.0, 0.5]),
    ([1.0, 1.0, 1.0], [1.0, 0.0]),
    ([-1.0, -1.0, 1.0], [0.5, 0.5]),
    ([1.0, -1.0, 1.0], [1.0, 0.5]),
    ([-1.0, 1.0, 1.0], [1.0, 0.0]),
    ([-1.0, -1.0, -1.0], [0.5, 0.5]),
    ([-1.0, -1.0, 1.0], [1.0, 0.5]),
    ([-1.0, 1.0, -1.0], [1.0, 0.0]),
    ([1.0, -1.0, -1.0], [0.5, 0.5]),
    ([-1.0, -1.0, -1.0], [1.0, 0.5]),
    ([-1.0, -1.0, 1.0], [0.5, 1.0]),
    ([1.0, -1.0, -1.0], [0.0, 0.5]),
    ([1.0, -1.0, 1.0], [0.0, 1.0]),
    ([1.0, 1.0, 1.0], [0.5, 0.5]),
    ([-1.0, 1.0, -1.0], [0.0, 0.0]),
    ([-1.0, 1.0, 1.0], [0.0, 0.5]),
    ([1.0, 1.0, 1.0], [0.5, 0.0]),
    ([-1.0, 1.0, 1.0], [0.5, 0.0]),
    ([-1.0, 1.0, -1.0], [0.5, 0.0]),
    ([1.0, 1.0, -1.0], [0.5, 0.0]),
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
