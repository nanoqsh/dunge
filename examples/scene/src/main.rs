mod models;

use {
    dunge::{
        color::Srgba,
        handles::*,
        input::{Input, Key},
        topology::LineStrip,
        transform::{Position, ReverseRotation, Transform},
        vertex::{ColorVertex, TextureVertex},
        Compare, Context, Error, Frame, FrameParameters, InitialState, Loop, MeshData,
        Orthographic, PixelSize, Source, SpaceData, TextureData, View, WindowMode,
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
        show_cursor: false,
        ..Default::default()
    })
    .run_blocking(App::new);
}

struct Model {
    instance: InstanceHandle,
    mesh: MeshHandle<TextureVertex>,
    update_view: bool,
    pos: [f32; 3],
}

struct Cube {
    instance: InstanceHandle,
    mesh: MeshHandle<ColorVertex, LineStrip>,
}

struct App {
    texture_layer: LayerHandle<TextureVertex>,
    color_layer: LayerHandle<ColorVertex, LineStrip>,
    sprites: TextureHandle,
    models: Vec<Model>,
    cubes: Vec<Cube>,
    light: LightHandle,
    lightspace: SpaceHandle,
    view: ViewHandle,
    camera: Camera,
    time: f32,
    fullscreen: bool,
}

impl App {
    fn new(context: &mut Context) -> Self {
        context.set_frame_parameters(FrameParameters {
            pixel_size: PixelSize::X1,
            ..Default::default()
        });

        // Create layer
        let texture_layer = context.create_layer();
        let color_layer = context
            .create_layer_with_parameters()
            .with_depth_compare(Compare::Always)
            .build();

        // Create the sprite texture
        let sprites = {
            let image = utils::read_png(include_bytes!("sprites.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context.create_texture(data)
        };

        // Create the light space
        let lightspace = {
            let layers = [
                utils::read_png(include_bytes!("lightmap_side.png")),
                utils::read_png(include_bytes!("lightmap_center.png")),
                utils::read_png(include_bytes!("lightmap_side.png")),
            ];

            let mut map = vec![];
            for layer in &layers {
                map.extend_from_slice(&layer);
            }

            let size = {
                let (width, height) = layers[0].dimensions();
                (width as u8, height as u8, layers.len() as u8)
            };

            let model = Transform {
                pos: [0., 0., 0.],
                scl: [1.; 3],
                ..Default::default()
            };

            let data = SpaceData::new(&map, size).expect("create space");
            context.create_space(model, data)
        };

        // Create models
        #[allow(clippy::needless_range_loop)]
        let models = {
            const D: u8 = 0;
            const E: u8 = 1;
            const W: u8 = 2;
            const V: u8 = 3;
            const F: u8 = 4;
            const L: u8 = 5;

            const N: usize = 9;
            const SCENE: [[u8; N]; N] = [
                [0, 0, W, V, W, V, V, 0, 0],
                [0, 0, W, L, F, L, V, 0, 0],
                [V, W, W, F, L, F, W, W, W],
                [F, L, L, F, F, F, L, L, W],
                [V, W, L, F, L, F, F, W, W],
                [0, W, L, W, L, L, L, W, 0],
                [0, V, L, L, L, F, L, W, 0],
                [0, V, W, W, F, W, W, V, 0],
                [0, 0, 0, V, L, V, 0, 0, 0],
            ];

            let meshes = [
                (models::diamond::VERTICES, models::diamond::INDICES),
                (models::enemy::VERTICES, models::enemy::INDICES),
                (models::wall::VERTICES, models::wall::INDICES),
                (models::wall_light::VERTICES, models::wall_light::INDICES),
                (models::floor::VERTICES, models::floor::INDICES),
                (models::floor_dark::VERTICES, models::floor_dark::INDICES),
            ];

            let mesh_handles: Vec<_> = meshes
                .into_iter()
                .map(|(verts, indxs)| {
                    let verts: Vec<_> = verts
                        .iter()
                        .map(|&(pos, map)| TextureVertex { pos, map })
                        .collect();

                    let indxs = indxs.to_vec();

                    let data = MeshData::new(&verts, &indxs).expect("create mesh");
                    context.create_mesh(&data)
                })
                .collect();

            let mut model_data = vec![];

            for z in 0..N {
                for x in 0..N {
                    let mut obj = SCENE[z][x];
                    if obj == 0 {
                        continue;
                    }

                    let x = x as f32 - N as f32 * 0.5 + 0.5;
                    let z = z as f32 - N as f32 * 0.5 + 0.5;
                    let y = f32::from(u8::from(obj == F || obj == L)) * -0.5;

                    if obj == V {
                        model_data.push((obj, [x, y + 1., z]));
                        obj = W;
                    }

                    model_data.push((obj, [x, y, z]));
                }
            }

            model_data.extend([
                (E, [1., 0., 0.]),
                (E, [-1., 0., -1.]),
                (D, [-2., 0., 2.]),
                (D, [-2., 0., 1.]),
                (D, [-1., 0., 2.]),
            ]);

            model_data
                .into_iter()
                .map(|(n, pos)| {
                    let mesh = mesh_handles[n as usize];
                    let instance = context.create_instances([Position(pos)]);
                    Model {
                        instance,
                        mesh,
                        update_view: n == D || n == E,
                        pos,
                    }
                })
                .collect()
        };

        // Create cube models
        let cubes = {
            const POSITIONS: [[f32; 3]; 2] = [[1., 0., 0.], [-1., 0., -1.]];

            POSITIONS
                .into_iter()
                .map(|pos| Cube {
                    instance: context.create_instances([Position(pos)]),
                    mesh: {
                        let verts: Vec<_> = models::square::VERTICES
                            .iter()
                            .map(|&pos| ColorVertex {
                                pos,
                                col: [0., 1., 0.3],
                            })
                            .collect();

                        let data = MeshData::from_verts(&verts);
                        context.create_mesh(&data)
                    },
                })
                .collect()
        };

        // Crate the light
        let light = context.create_light([0.; 3], []).expect("create light");

        // Create the view
        let camera = Camera::default();
        let view = context.create_view(camera.view(Orthographic::default()));

        Self {
            texture_layer,
            color_layer,
            sprites,
            models,
            cubes,
            light,
            lightspace,
            view,
            camera,
            time: 0.,
            fullscreen: false,
        }
    }
}

impl Loop for App {
    type Error = Error;

    fn update(&mut self, context: &mut Context, input: &Input) -> Result<(), Self::Error> {
        use {dunge::winit::window::Fullscreen, std::f32::consts::TAU};

        const SENSITIVITY: f32 = 0.01;
        const AMBIENT_COLOR: [f32; 3] = [0.4; 3];
        const LIGHTS_DISTANCE: f32 = 3.3;
        const LIGHTS_SPEED: f32 = 1.;
        const INTENSITY: f32 = 0.;
        const LIGHTS: [(f32, [f32; 3]); 3] = [
            (0., [INTENSITY, 0., 0.]),
            (TAU / 3., [0., INTENSITY, 0.]),
            (TAU * 2. / 3., [0., 0., INTENSITY]),
        ];

        self.time += input.delta_time * LIGHTS_SPEED;
        let make_source = |step, col| Source {
            pos: {
                let step: f32 = (self.time + step) % TAU;
                [
                    step.sin() * LIGHTS_DISTANCE,
                    0.,
                    step.cos() * LIGHTS_DISTANCE,
                ]
            },
            rad: 3.,
            col,
            ..Default::default()
        };

        context.update_light(
            self.light,
            AMBIENT_COLOR,
            LIGHTS.map(|(step, col)| make_source(step, col)),
        )?;

        // Handle pressed keys
        for key in input.pressed_keys {
            match key {
                Key::Escape => {
                    context.plan_to_close();
                    return Ok(());
                }
                Key::P => {
                    let shot = context.take_screenshot();
                    utils::create_image(shot.width, shot.height, shot.data)
                        .save("screen.png")
                        .expect("save screenshot");
                }
                Key::F1 => {
                    self.fullscreen = !self.fullscreen;
                    context
                        .window()
                        .set_fullscreen(self.fullscreen.then_some(Fullscreen::Borderless(None)));
                }
                _ => (),
            }
        }

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let sprite_scale = 16. * 2.;
        let view = View {
            proj: Orthographic {
                width_factor: 1. / sprite_scale,
                height_factor: 1. / sprite_scale,
                ..Default::default()
            },
            ..self.camera.view(Orthographic::default())
        };
        context.update_view(self.view, view)?;

        self.models
            .iter()
            .filter(|model| model.update_view)
            .try_for_each(|model| {
                context.update_instances(
                    model.instance,
                    [Transform {
                        pos: model.pos,
                        rot: ReverseRotation(view),
                        scl: [1.; 3],
                    }],
                )
            })?;

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        const CLEAR_COLOR: Srgba<u8> = Srgba([46, 34, 47, 255]);

        {
            let mut layer = frame
                .layer(self.texture_layer)?
                .with_clear_color(CLEAR_COLOR)
                .with_clear_depth()
                .start();

            layer.bind_light(self.light)?;
            layer.bind_space(self.lightspace)?;
            layer.bind_view(self.view)?;
            layer.bind_texture(self.sprites)?;
            for model in &self.models {
                layer.bind_instance(model.instance)?;
                layer.draw(model.mesh)?;
            }
        }

        {
            let mut layer = frame.layer(self.color_layer)?.start();
            layer.bind_view(self.view)?;
            for cube in &self.cubes {
                layer.bind_instance(cube.instance)?;
                // layer.draw(cube.mesh)?;
            }
        }

        Ok(())
    }
}
