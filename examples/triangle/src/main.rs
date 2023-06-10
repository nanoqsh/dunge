use {
    dunge::{
        color::Standard,
        handles::*,
        input::{Input, Key},
        transform::Position,
        CanvasConfig, Context, Error, Frame, InitialState, Loop, MeshData, Shader, TextureData,
        Vertex, WindowMode,
    },
    dunge_shader::{Dimension, Fragment, Scheme, Vertex as SchemeVertex, View},
};

#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3], #[texture] [f32; 2]);

struct ColorShader;
impl Shader for ColorShader {
    type Vertex = Vert;
    const VIEW: View = View::Camera;
    const AMBIENT: bool = true;
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
    textures: TexturesHandle<ColorShader>,
    mesh: MeshHandle<Vert>,
    instance: InstanceHandle,
    state: f32,
}

impl App {
    fn new(context: &mut Context) -> Self {
        // Create shader and layer
        let layer = {
            let shader: ShaderHandle<ColorShader> = context.create_shader(Scheme {
                vert: SchemeVertex {
                    dimension: Dimension::D2,
                    fragment: Fragment {
                        vertex_color: true,
                        vertex_texture: true,
                    },
                },
                view: View::Camera,
                static_color: None,
                ambient: true,
            });

            context
                .create_layer_with_parameters()
                .with_cull_faces(false)
                .build(shader)
                .expect("create layer")
        };

        // Create globals
        let globals = context
            .globals_builder()
            .with_view()
            .with_ambient(Standard([0.1, 0.3, 1.]))
            .build(layer)
            .expect("create globals");

        // Create textures
        let textures = {
            let im = utils::read_png(include_bytes!("../nana.png"));
            let data = TextureData::new(&im, im.dimensions()).expect("texture data");

            context
                .textures_builder()
                .with_map(data)
                .build(layer)
                .expect("create textures")
        };

        // Create a mesh
        let mesh = {
            const SIZE: f32 = 160.;
            const VERTICES: [Vert; 3] = [
                Vert([-SIZE, -SIZE], [1., 0., 0.], [0., 1.]),
                Vert([SIZE, -SIZE], [0., 1., 0.], [1., 1.]),
                Vert([0., SIZE], [0., 0., 1.], [0.5, 0.]),
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
            textures,
            mesh,
            instance,
            state: 0.,
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

        self.state += input.delta_time;
        context
            .update_globals_view(
                self.globals,
                dunge::View {
                    up: [self.state.sin(), self.state.cos(), 0.],
                    ..dunge::View::default()
                },
            )
            .expect("update globals");

        context
            .update_globals_ambient(
                self.globals,
                Standard([self.state.sin() * 0.5 + 1., self.state.cos() * 0.5 + 1., 1.]),
            )
            .expect("update globals");

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        frame
            .layer(self.layer)?
            .with_clear_color(Standard([0, 0, 0, u8::MAX]))
            .with_clear_depth()
            .start()
            .bind_globals(self.globals)?
            .bind_textures(self.textures)?
            .bind_instance(self.instance)?
            .draw(self.mesh)
    }
}
