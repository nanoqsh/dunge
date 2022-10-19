use {
    dunge::{
        color::Srgba, input::Input, transform::Position, Context, Error, Frame, InstanceHandle,
        Loop, MeshData, MeshHandle, Perspective, TextureData, TextureHandle, TextureVertex,
        ViewHandle,
    },
    utils::Camera,
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen(start)]
pub async fn run() {
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));
    dunge::from_element("root").run(App::new).await;
}

struct App {
    texture: TextureHandle,
    instance: InstanceHandle,
    mesh: MeshHandle<TextureVertex>,
    view: ViewHandle,
    camera: Camera,
}

impl App {
    fn new(context: &mut Context) -> Self {
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

        // Create a mesh
        let mesh = {
            let verts = VERTICES.map(|(pos, map)| TextureVertex { pos, map });
            let data = MeshData::new(&verts, &INDICES).expect("create mesh");
            context.create_mesh(data)
        };

        // Create the view
        let camera = Camera::default();
        let view = context.create_view(camera.view::<Perspective>());

        Self {
            texture,
            instance,
            mesh,
            view,
            camera,
        }
    }
}

impl Loop for App {
    type Error = Error;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        const SENSITIVITY: f32 = 0.01;

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let view = self.camera.view::<Perspective>();
        context.update_view(self.view, view)?;

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        let mut layer = frame.start_texture_layer(Srgba([29, 39, 34, 255]));

        layer.bind_view(self.view)?;
        layer.bind_texture(self.texture)?;
        layer.bind_instance(self.instance)?;
        layer.draw(self.mesh)?;

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
