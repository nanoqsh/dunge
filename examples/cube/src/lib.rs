mod data;

use {
    dunge::{
        input::Key, shader::*, Context, Frame, Globals, Input, Instance, Layer, Loop, Mesh,
        MeshData, ModelTransform, Perspective, Rgba, TextureData, Textures, Vertex, View,
    },
    utils::Camera,
};

#[repr(C)]
#[derive(Vertex)]
struct TextureVert {
    #[position]
    pos: [f32; 3],
    #[texture]
    map: [f32; 2],
}

struct TextureShader;

impl Shader for TextureShader {
    type Vertex = TextureVert;
    const VIEW: ShaderView = ShaderView::Camera;
}

#[repr(C)]
#[derive(Vertex)]
struct ColorVert {
    #[position]
    pos: [f32; 3],
    #[color]
    col: [f32; 3],
}

struct ColorShader;

impl Shader for ColorShader {
    type Vertex = ColorVert;
    const VIEW: ShaderView = ShaderView::Camera;
}

enum State {
    Texture,
    Color,
}

pub struct App {
    texture_layer: Layer<TextureShader>,
    color_layer: Layer<ColorShader>,
    texture_globals: Globals<TextureShader>,
    color_globals: Globals<ColorShader>,
    textures: Textures<TextureShader>,
    instance: Instance,
    texture_mesh: Mesh<TextureVert>,
    color_mesh: Mesh<ColorVert>,
    camera: Camera,
    state: State,
}

impl App {
    pub fn new(context: &mut Context) -> Self {
        // Create layers.
        let texture_layer = context.create_layer();
        let color_layer = context.create_layer();

        // Create globals
        let texture_globals = context
            .globals_builder()
            .with_view(View::default())
            .build(&texture_layer);

        let color_globals = context
            .globals_builder()
            .with_view(View::default())
            .build(&color_layer);

        // Create a textures
        let textures = {
            let image = utils::read_png(include_bytes!("grass.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context
                .textures_builder()
                .with_map(data)
                .build(&texture_layer)
        };

        // Create a model instance
        let instance = context.create_instances(&[ModelTransform::default()]);

        // Create meshes
        let texture_mesh = {
            let verts = data::VERTICES.map(|(pos, map)| TextureVert { pos, map });

            let data = MeshData::new(&verts, &data::INDICES).expect("create mesh");
            context.create_mesh(&data)
        };

        let color_mesh = {
            let verts = data::VERTICES.map(|(pos, [a, b])| ColorVert {
                pos,
                col: [a, b, 1.],
            });

            let data = MeshData::new(&verts, &data::INDICES).expect("create mesh");
            context.create_mesh(&data)
        };

        Self {
            texture_layer,
            color_layer,
            texture_globals,
            color_globals,
            textures,
            instance,
            texture_mesh,
            color_mesh,
            camera: Camera::default(),
            state: State::Texture,
        }
    }
}

impl Loop for App {
    fn update(&mut self, context: &mut Context, input: &Input) {
        const SENSITIVITY: f32 = 0.01;

        // Handle pressed keys
        for key in input.pressed_keys {
            match key {
                Key::Escape => context.plan_to_close(),
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
        self.texture_globals.update_view(view);
        self.color_globals.update_view(view);
    }

    fn render(&self, frame: &mut Frame) {
        let clear_color = Rgba::from_standard_bytes([46, 34, 47, 255]);
        match self.state {
            State::Texture => frame
                .layer(&self.texture_layer)
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start()
                .bind_globals(&self.texture_globals)
                .bind_textures(&self.textures)
                .draw(&self.texture_mesh, &self.instance),
            State::Color => frame
                .layer(&self.color_layer)
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start()
                .bind_globals(&self.color_globals)
                .draw(&self.color_mesh, &self.instance),
        }
    }
}
