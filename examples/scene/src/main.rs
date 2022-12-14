mod models;

use {
    dunge::{
        color::Srgba,
        input::{Input, Key},
        transform::{Position, ReverseRotation, Transform},
        vertex::TextureVertex,
        Context, Error, Frame, FrameParameters, InitialState, InstanceHandle, Loop, MeshData,
        MeshHandle, Orthographic, TextureData, TextureHandle, View, ViewHandle, WindowMode,
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

struct App {
    sprites: TextureHandle,
    models: Vec<Model>,
    view: ViewHandle,
    camera: Camera,
    antialiasing: bool,
}

impl App {
    fn new(context: &mut Context) -> Self {
        context.set_frame_parameters(FrameParameters {
            pixel_size: 2,
            ..Default::default()
        });

        // Create the sprite texture
        let sprites = {
            let image = utils::read_png(include_bytes!("sprites.png"));
            let data = TextureData::new(&image, image.dimensions()).expect("create texture");
            context.create_texture(data)
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

        // Create the view
        let camera = Camera::default();
        let view = context.create_view(camera.view::<Orthographic>());

        Self {
            sprites,
            models,
            view,
            camera,
            antialiasing: false,
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
                Key::Space => self.antialiasing = !self.antialiasing,
                _ => {}
            }
        }

        let (x, z) = input.mouse.motion_delta;
        let (_, y) = input.mouse.wheel_delta;

        self.camera.update((x * SENSITIVITY, y, z * SENSITIVITY));

        // Set the view
        let sprite_scale = 16.;
        let view = View {
            proj: Orthographic {
                width_factor: 1. / sprite_scale,
                height_factor: 1. / sprite_scale,
                ..Default::default()
            },
            ..self.camera.view()
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
        let mut layer = frame
            .texture_layer()
            .with_clear_color(Srgba([46, 34, 47, 255]))
            .with_clear_depth()
            .start();

        layer.bind_view(self.view)?;
        layer.bind_texture(self.sprites)?;
        for model in &self.models {
            layer.bind_instance(model.instance)?;
            layer.draw(model.mesh)?;
        }

        Ok(())
    }
}
