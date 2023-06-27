use {
    dunge::{
        color::Standard,
        handles::*,
        input::{Input, Key},
        transform::Position,
        CanvasConfig, Context, Error, Frame, FrameParameters, InitialState, Loop, MeshData,
        PixelSize, Shader, Source, TextureData, Vertex, WindowMode,
    },
    dunge_shader::{
        Dimension, Fragment, Scheme, SourceArray, SourceArrays, SourceKind, Vertex as SchemeVertex,
        View,
    },
};

#[repr(C)]
#[derive(Vertex)]
struct Vert(#[position] [f32; 2], #[color] [f32; 3], #[texture] [f32; 2]);

struct TriangleShader;
impl Shader for TriangleShader {
    type Vertex = Vert;
    const VIEW: View = View::Camera;
    const AMBIENT: bool = true;
    const SOURCES: SourceArrays = SourceArrays::new(&SOURCE_ARRAYS);
}

const SOURCE_ARRAYS: [SourceArray; 1] = [SourceArray::new(SourceKind::Glow, 4)];

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
    layer: LayerHandle<TriangleShader>,
    globals: GlobalsHandle<TriangleShader>,
    textures: TexturesHandle<TriangleShader>,
    lights: LightsHandle<TriangleShader>,
    mesh: MeshHandle<Vert>,
    instance: InstanceHandle,
    state: f32,
}

impl App {
    fn new(context: &mut Context) -> Self {
        context.set_frame_parameters(FrameParameters {
            pixel_size: PixelSize::X1,
            ..Default::default()
        });

        // Create shader and layer
        let layer = {
            let shader: ShaderHandle<TriangleShader> = context.create_shader(Scheme {
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
                source_arrays: SourceArrays::new(&SOURCE_ARRAYS),
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
            .with_ambient(Standard([0.5; 3]))
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

        // Create lights
        let lights = {
            context
                .lights_builder()
                .with_sources(&[])
                .build(layer)
                .expect("create lights")
        };

        // Create a mesh
        let mesh = {
            const SIZE: f32 = 160.;
            const VERTICES: [Vert; 3] = [
                Vert([-SIZE, -SIZE], [1.; 3], [0., 1.]),
                Vert([SIZE, -SIZE], [1.; 3], [1., 1.]),
                Vert([0., SIZE], [1.; 3], [0.5, 0.]),
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
            lights,
            mesh,
            instance,
            state: 0.,
        }
    }
}

impl Loop for App {
    type Error = Error;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        use std::f32::consts::TAU;

        // Handle pressed keys
        for key in input.pressed_keys {
            if key == Key::Escape {
                context.plan_to_close();
                return Ok(());
            }

            if key == Key::P {
                let shot = context.take_screenshot();
                utils::create_image(shot.width, shot.height, shot.data)
                    .save("screen.png")
                    .expect("save screenshot");
            }
        }

        self.state += input.delta_time * 0.5;

        let size = 80.;
        let step = TAU / 4.;
        let calc_position = |i: f32| {
            let state = self.state + step * i;
            [state.sin() * size, state.cos() * size, 0.]
        };

        context
            .update_lights_sources(
                self.lights,
                0,
                &[
                    Source {
                        col: [0., 2., 0.],
                        pos: calc_position(1.),
                        rad: 80.,
                    },
                    Source {
                        col: [2., 0., 0.],
                        pos: calc_position(2.),
                        rad: 80.,
                    },
                    Source {
                        col: [0., 0., 2.],
                        pos: calc_position(3.),
                        rad: 80.,
                    },
                    Source {
                        col: [2., 2., 0.],
                        pos: calc_position(4.),
                        rad: 80.,
                    },
                ],
            )
            .expect("update lights");

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
            .bind_lights(self.lights)?
            .bind_instance(self.instance)?
            .draw(self.mesh)
    }
}
