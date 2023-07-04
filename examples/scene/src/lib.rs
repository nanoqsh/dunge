mod models;

use {
    dunge::{
        handles::*,
        input::{Input, Key},
        shader::*,
        topology::LineStrip,
        Color, Compare, Context, Error, Frame, FrameParameters, Loop, MeshData, Model,
        Orthographic, PixelSize, Rgb, Rgba, Source, Space, SpaceData, SpaceFormat, TextureData,
        Transform, Vertex,
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
    const AMBIENT: bool = true;
    const SOURCES: SourceArrays = SourceArrays::new(&[SourceArray::new(SourceKind::Glow, 3)]);
    const SPACES: LightSpaces = LightSpaces::new(&[SpaceKind::Rgba]);
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

struct Mesh {
    instance: InstanceHandle,
    mesh: MeshHandle<TextureVert>,
    update_view: bool,
    pos: [f32; 3],
}

struct Cube {
    instance: InstanceHandle,
    mesh: MeshHandle<ColorVert, LineStrip>,
}

pub struct App {
    texture_layer: LayerHandle<TextureShader>,
    color_layer: LayerHandle<ColorShader, LineStrip>,
    sprites: TexturesHandle<TextureShader>,
    meshes: Vec<Mesh>,
    cubes: Vec<Cube>,
    texture_globals: GlobalsHandle<TextureShader>,
    color_globals: GlobalsHandle<ColorShader>,
    lights: LightsHandle<TextureShader>,
    spaces: SpacesHandle<TextureShader>,
    camera: Camera,
    time: f32,
    fullscreen: bool,
}

impl App {
    pub fn new(context: &mut Context) -> Self {
        const AMBIENT_COLOR: Rgb = Color([0.09; 3]);

        context.set_frame_parameters(FrameParameters {
            pixel_size: PixelSize::X2,
            ..Default::default()
        });

        // Create shaders and layers
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
                .with_depth_compare(Compare::Always)
                .build(shader)
                .expect("create color layer")
        };

        // Create the sprite texture
        let sprites = {
            let image = utils::read_png(include_bytes!("sprites.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context
                .textures_builder()
                .with_map(data)
                .build(texture_layer)
                .expect("create textures")
        };

        // Create globals
        let texture_globals = context
            .globals_builder()
            .with_view()
            .with_ambient(AMBIENT_COLOR)
            .build(texture_layer)
            .expect("create texture globals");

        let color_globals = context
            .globals_builder()
            .with_view()
            .build(color_layer)
            .expect("create color globals");

        // Crate the lights
        let lights = context
            .lights_builder()
            .with_sources(vec![])
            .build(texture_layer)
            .expect("create light");

        // Create the light spaces
        let spaces = {
            let layers = [
                utils::read_png(include_bytes!("lightmap_side.png")),
                utils::read_png(include_bytes!("lightmap_center.png")),
                utils::read_png(include_bytes!("lightmap_side.png")),
            ];

            let mut map = vec![];
            for layer in &layers {
                map.extend_from_slice(layer);
            }

            let size = {
                let (width, height) = layers[0].dimensions();
                (width as u8, height as u8, layers.len() as u8)
            };

            let space = Space {
                data: SpaceData::new(&map, size, SpaceFormat::Srgba).expect("create space"),
                model: Model::default(),
                col: Color([2.5; 3]),
            };

            context
                .spaces_builder()
                .with_space(space)
                .build(texture_layer)
                .expect("create space")
        };

        // Create models
        #[allow(clippy::needless_range_loop)]
        let meshes = {
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
                [0, V, W, W, L, W, W, V, 0],
                [0, 0, 0, V, F, V, 0, 0, 0],
            ];

            let vertices = [
                (models::diamond::VERTICES, models::diamond::INDICES),
                (models::enemy::VERTICES, models::enemy::INDICES),
                (models::wall::VERTICES, models::wall::INDICES),
                (models::wall_light::VERTICES, models::wall_light::INDICES),
                (models::floor::VERTICES, models::floor::INDICES),
                (models::floor_dark::VERTICES, models::floor_dark::INDICES),
            ];

            let mesh_handles: Vec<_> = vertices
                .into_iter()
                .map(|(verts, indxs)| {
                    let verts: Vec<_> = verts
                        .iter()
                        .map(|&(pos, map)| TextureVert { pos, map })
                        .collect();

                    let indxs = indxs.to_vec();
                    let data = MeshData::new(&verts, &indxs).expect("create mesh");
                    context.create_mesh(&data)
                })
                .collect();

            let mut mesh_data = vec![];
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
                        mesh_data.push((obj, [x, y + 1., z]));
                        obj = W;
                    }

                    mesh_data.push((obj, [x, y, z]));
                }
            }

            mesh_data.extend([
                (E, [1., 0., 0.]),
                (E, [-1., 0., -1.]),
                (D, [-2., 0., 2.]),
                (D, [-2., 0., 1.]),
                (D, [-1., 0., 2.]),
            ]);

            mesh_data
                .into_iter()
                .map(|(n, pos)| {
                    let transform = Transform::from_position(pos);
                    Mesh {
                        instance: context.create_instances(&[Model::from(transform)]),
                        mesh: mesh_handles[n as usize],
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
                .map(|pos| {
                    let transform = Transform::from_position(pos);
                    Cube {
                        instance: context.create_instances(&[Model::from(transform)]),
                        mesh: {
                            let verts: Vec<_> = models::square::VERTICES
                                .iter()
                                .map(|&pos| ColorVert {
                                    pos,
                                    col: [0., 1., 0.3],
                                })
                                .collect();

                            let data = MeshData::from_verts(&verts);
                            context.create_mesh(&data)
                        },
                    }
                })
                .collect()
        };

        Self {
            texture_layer,
            color_layer,
            sprites,
            meshes,
            cubes,
            texture_globals,
            color_globals,
            lights,
            spaces,
            camera: Camera::default(),
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
        const LIGHTS_DISTANCE: f32 = 3.3;
        const LIGHTS_SPEED: f32 = 1.;
        const INTENSITY: f32 = 2.;
        const LIGHTS: [(f32, [f32; 3]); 3] = [
            (0., [INTENSITY, 0., 0.]),
            (TAU / 3., [0., INTENSITY, 0.]),
            (TAU * 2. / 3., [0., 0., INTENSITY]),
        ];

        self.time += input.delta_time * LIGHTS_SPEED;
        let make_source = |step, col| {
            let step = (self.time + step) % TAU;
            Source::new(
                Color(col),
                [
                    f32::sin(step) * LIGHTS_DISTANCE,
                    0.,
                    f32::cos(step) * LIGHTS_DISTANCE,
                ],
                2.,
            )
        };

        context
            .update_lights_sources(
                self.lights,
                0,
                &LIGHTS.map(|(step, col)| make_source(step, col)),
            )
            .expect("update lights");

        // Handle pressed keys
        #[cfg(not(target_arch = "wasm32"))]
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
                _ => {}
            }
        }

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let sprite_scale = 4. * 6.;
        let view = self.camera.view(Orthographic {
            width_factor: 1. / sprite_scale,
            height_factor: 1. / sprite_scale,
            ..Default::default()
        });

        context.update_globals_view(self.texture_globals, view)?;
        context.update_globals_view(self.color_globals, view)?;

        self.meshes
            .iter()
            .filter(|mesh| mesh.update_view)
            .try_for_each(|mesh| {
                let transform = Transform {
                    pos: mesh.pos.into(),
                    rot: view.rotation().conjugate(),
                    ..Default::default()
                };

                context.update_instances(mesh.instance, &[Model::from(transform)])
            })?;

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        {
            let clear_color = Rgba::from_standard_bytes([46, 34, 47, 255]);
            let mut layer = frame
                .layer(self.texture_layer)?
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start();

            layer
                .bind_globals(self.texture_globals)?
                .bind_lights(self.lights)?
                .bind_spaces(self.spaces)?
                .bind_textures(self.sprites)?;

            for model in &self.meshes {
                layer.draw(model.mesh, model.instance)?;
            }
        }

        {
            let mut layer = frame.layer(self.color_layer)?.start();
            layer.bind_globals(self.color_globals)?;
            for cube in &self.cubes {
                layer.draw(cube.mesh, cube.instance)?;
            }
        }

        Ok(())
    }
}
