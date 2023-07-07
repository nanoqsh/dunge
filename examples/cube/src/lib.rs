mod data;

use {
    dunge::{
        handles::*,
        input::{Input, Key},
        shader::*,
        Context, Error, Frame, Loop, Mesh, MeshData, Model, Perspective, Rgba, TextureData, Vertex,
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
    texture_layer: LayerHandle<TextureShader>,
    color_layer: LayerHandle<ColorShader>,
    texture_globals: GlobalsHandle<TextureShader>,
    color_globals: GlobalsHandle<ColorShader>,
    textures: TexturesHandle<TextureShader>,
    instance: InstanceHandle,
    texture_mesh: Mesh<TextureVert>,
    color_mesh: Mesh<ColorVert>,
    camera: Camera,
    state: State,
}

impl App {
    pub fn new(context: &mut Context) -> Self {
        // Create layers. The vertex type inferred from the context
        let texture_layer = {
            let shader: ShaderHandle<TextureShader> = context.create_shader();
            context
                .create_layer_with_parameters()
                .build(shader)
                .expect("create texture layer")
        };

        let color_layer = {
            let shader: ShaderHandle<ColorShader> = context.create_shader();
            context
                .create_layer_with_parameters()
                .build(shader)
                .expect("create color layer")
        };

        // Create globals
        let texture_globals = context
            .globals_builder()
            .with_view()
            .build(texture_layer)
            .expect("create texture globals");

        let color_globals = context
            .globals_builder()
            .with_view()
            .build(color_layer)
            .expect("create color globals");

        // Create a textures
        let textures = {
            let image = utils::read_png(include_bytes!("grass.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context
                .textures_builder()
                .with_map(data)
                .build(texture_layer)
                .expect("create textures")
        };

        // Create a model instance
        let instance = context.create_instances(&[Model::default()]);

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
        context.update_globals_view(self.texture_globals, view)?;
        context.update_globals_view(self.color_globals, view)?;
        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        let clear_color = Rgba::from_standard_bytes([46, 34, 47, 255]);
        match self.state {
            State::Texture => frame
                .layer(self.texture_layer)?
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start()
                .bind_globals(self.texture_globals)?
                .bind_textures(self.textures)?
                .draw(&self.texture_mesh, self.instance)?,
            State::Color => frame
                .layer(self.color_layer)?
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start()
                .bind_globals(self.color_globals)?
                .draw(&self.color_mesh, self.instance)?,
        }

        Ok(())
    }
}
