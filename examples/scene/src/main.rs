mod models;

use {
    dunge::{
        color::Srgba,
        input::{Input, Key},
        transform::{Inversed, Position, Transform},
        Context, Error, Frame, InitialState, InstanceHandle, Loop, MeshData, MeshHandle,
        TextureData, TextureHandle, TextureVertex, WindowMode,
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
        ..InitialState::default()
    })
    .run_blocking(App::new);
}

struct Model {
    instance: InstanceHandle,
    mesh: MeshHandle,
    update_view: bool,
    pos: [f32; 3],
}

struct App {
    sprites: TextureHandle,
    models: Vec<Model>,
    camera: Camera,
}

impl App {
    fn new(context: &mut Context) -> Self {
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
                [0, 0, 0, 0, W, W, W, 0, 0],
                [0, 0, W, W, W, L, W, 0, 0],
                [V, W, W, F, L, F, W, W, V],
                [F, L, L, F, F, F, L, L, V],
                [V, W, L, F, L, F, F, V, V],
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
                    context.create_mesh(data)
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
                    let y = (obj == F || obj == L) as u8 as f32 * -0.5;

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

        // Set the clear color
        let color = Srgba([46, 34, 47, 255]);
        context.set_clear_color(color);

        // Create the camera
        let camera = Camera::default();

        Self {
            sprites,
            models,
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
            if key == Key::Escape {
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

        self.models
            .iter()
            .filter(|model| model.update_view)
            .try_for_each(|model| {
                context.update_instances(
                    model.instance,
                    [Transform {
                        pos: model.pos,
                        rot: Inversed(view),
                        scl: [1.; 3],
                    }],
                )
            })?;

        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), Self::Error> {
        frame.bind_texture(self.sprites)?;
        for model in &self.models {
            frame.set_instance(model.instance)?;
            frame.draw_mesh(model.mesh)?;
        }

        Ok(())
    }
}
