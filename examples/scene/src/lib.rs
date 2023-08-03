mod atlas;
mod models;

use {
    crate::atlas::{Atlas, Rect},
    dunge::{
        input::Key, shader::*, topology::LineStrip, Blend, Color, Compare, Context, Format, Frame,
        FrameParameters, Globals, Input, Instance, InstanceColor, Layer, Lights, Loop, Mesh,
        MeshData, ModelColor, ModelTransform, Orthographic, PixelSize, PostEffect, Rgb, Rgba,
        Source, Space, SpaceData, Spaces, TextureData, Textures, Transform, Vertex, View,
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
    const TEXTURES: TexturesNumber = TexturesNumber::N1.with_discard_threshold(0.9);
    const SOURCES: SourceArrays = SourceArrays::new(&[SourceArray::new(SourceKind::Glow, 3)]);
    const SPACES: LightSpaces = LightSpaces::new(&[SpaceKind::Rgba]);
}

#[repr(C)]
#[derive(Vertex)]
struct ColorVert {
    #[position]
    pos: [f32; 3],
}

struct ColorShader;

impl Shader for ColorShader {
    type Vertex = ColorVert;
    const VIEW: ShaderView = ShaderView::Camera;
    const INSTANCE_COLORS: bool = true;
}

#[repr(C)]
#[derive(Clone, Copy, Default, Vertex)]
struct FontVert {
    #[position]
    pos: [f32; 2],
    #[texture]
    map: [f32; 2],
}

struct FontShader;

impl Shader for FontShader {
    type Vertex = FontVert;
    const TEXTURES: TexturesNumber = TexturesNumber::N1
        .with_gray_mode()
        .with_discard_threshold(0.95);
}

struct Sprite {
    instance: Instance,
    mesh: &'static Mesh<TextureVert>,
    update_view: bool,
    pos: [f32; 3],
}

struct Squares {
    instance: Instance,
    color: InstanceColor,
    mesh: Mesh<ColorVert, LineStrip>,
    len: usize,
}

struct Text {
    map: Textures<FontShader>,
    size: (u32, u32),
    atlas: Atlas,
    instance: Instance,
    mesh: Mesh<FontVert>,
    n: u32,
}

impl Text {
    const MAX_SYMBOLS: usize = 32;

    fn write(&mut self, s: &str, (sw, sh): (u32, u32)) {
        const PADDING: i32 = 16;
        const FONT_SIZE: i32 = 2;
        const SPACE_WIDTH: i32 = 4;

        let mut px = -(sw as i32 - PADDING);
        let py = sh as i32 - PADDING;
        let (mw, mh) = {
            let (mw, mh) = self.size;
            (mw as f32, mh as f32)
        };

        let (sw, sh) = (sw as f32, sh as f32);
        let vert = |x, y, u, v| FontVert {
            pos: [x, y],
            map: [u, v],
        };

        self.n = 0;
        let mut quads = Vec::with_capacity(Self::MAX_SYMBOLS * 4);
        for c in s.chars().take(Self::MAX_SYMBOLS) {
            self.n += 2;
            if c == ' ' {
                px += SPACE_WIDTH * FONT_SIZE + FONT_SIZE;
                continue;
            }

            let Rect { u, v, w, h } = self.atlas.get(c);
            let (x, y) = (px as f32 / sw, py as f32 / sh);
            let (dx, dy) = (
                w as f32 / sw * FONT_SIZE as f32,
                h as f32 / sh * FONT_SIZE as f32,
            );

            let (u, v) = (u as f32 / mw, v as f32 / mh);
            let (du, dv) = (w as f32 / mw, h as f32 / mh);
            quads.extend([
                vert(x, y, u, v),
                vert(x + dx, y, u + du, v),
                vert(x + dx, y - dy, u + du, v + dv),
                vert(x, y - dy, u, v + dv),
            ]);

            px += w as i32 * FONT_SIZE + FONT_SIZE;
        }

        self.mesh.update_verts(&quads).expect("update font mesh");
    }
}

pub struct App {
    texture_layer: Layer<TextureShader>,
    color_layer: Layer<ColorShader, LineStrip>,
    font_layer: Layer<FontShader>,
    texture_globals: Globals<TextureShader>,
    color_globals: Globals<ColorShader>,
    lights: Lights<TextureShader>,
    spaces: Spaces<TextureShader>,
    post: PostEffect,
    sprites: Textures<TextureShader>,
    sprite_meshes: Vec<Sprite>,
    squares: Squares,
    text: Text,
    camera: Camera,
    time_passed: f32,
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
        let texture_layer = context.create_layer();
        let color_layer = {
            let scheme = context.create_scheme();
            context
                .create_layer_with()
                .with_depth_compare(Compare::Always)
                .build(&scheme)
        };

        let font_layer = {
            let scheme = context.create_scheme();
            context
                .create_layer_with()
                .with_blend(Blend::AlphaBlending)
                .with_cull_faces(false)
                .with_depth_compare(Compare::Always)
                .build(&scheme)
        };

        // Create globals
        let texture_globals = context
            .globals_builder()
            .with_view(View::default())
            .with_ambient(AMBIENT_COLOR)
            .build(&texture_layer);

        let color_globals = context
            .globals_builder()
            .with_view(View::default())
            .build(&color_layer);

        let post = context
            .posteffect_builder()
            .vignette(Rgb::from_bytes([0; 3]), 0.3);

        // Crate the lights
        let lights = context
            .lights_builder()
            .with_sources(vec![])
            .build(&texture_layer);

        // Create the light spaces
        let spaces = {
            let layers = [
                utils::decode_rgba_png(include_bytes!("lightmap_side.png")),
                utils::decode_rgba_png(include_bytes!("lightmap_center.png")),
                utils::decode_rgba_png(include_bytes!("lightmap_side.png")),
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
                data: SpaceData::new(&map, size, Format::Srgba).expect("create space"),
                model: ModelTransform::default(),
                col: Color([2.5; 3]),
            };

            context
                .spaces_builder()
                .with_space(space)
                .build(&texture_layer)
        };

        // Create the sprite texture
        let sprites = {
            let image = utils::decode_rgba_png(include_bytes!("sprites.png"));
            let data = TextureData::new(&image, image.dimensions(), Format::Srgba)
                .expect("create texture");

            context
                .textures_builder()
                .with_map(data)
                .build(&texture_layer)
        };

        // Create models
        #[allow(clippy::needless_range_loop)]
        let sprite_meshes = {
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

            let meshes: Vec<&'static _> = vertices
                .into_iter()
                .map(|(verts, indxs)| {
                    let verts: Vec<_> = verts
                        .iter()
                        .map(|&(pos, map)| TextureVert { pos, map })
                        .collect();

                    let indxs = indxs.to_vec();
                    let data = MeshData::new(&verts, &indxs).expect("create mesh");
                    let mesh = context.create_mesh(&data);
                    Box::leak(mesh.into()) as &'static _
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
                    Sprite {
                        instance: context.create_instances(&[ModelTransform::from(transform)]),
                        mesh: meshes[n as usize],
                        update_view: n == D || n == E,
                        pos,
                    }
                })
                .collect()
        };

        // Create square models
        let squares = {
            const POSITIONS: [[f32; 3]; 2] = [[1., 0., 0.], [-1., 0., -1.]];

            let transforms: Vec<_> = POSITIONS
                .into_iter()
                .map(Transform::from_position)
                .map(ModelTransform::from)
                .collect();

            let colors = vec![ModelColor::default(); POSITIONS.len()];
            Squares {
                instance: context.create_instances(&transforms),
                color: context.create_instances_color(&colors),
                mesh: {
                    let verts: Vec<_> = models::square::VERTICES
                        .iter()
                        .map(|&pos| ColorVert { pos })
                        .collect();

                    let data = MeshData::from_verts(&verts);
                    context.create_mesh(&data)
                },
                len: POSITIONS.len(),
            }
        };

        // Create text
        let text = {
            let image = utils::decode_gray_png(include_bytes!("atlas.png"));
            let size = image.dimensions();
            let data = TextureData::new(&image, size, Format::Gray).expect("create atlas texture");

            let map = context.textures_builder().with_map(data).build(&font_layer);
            let atlas = serde_json::from_str(include_str!("atlas.json")).expect("read atlas map");
            let instance = context.create_instances(&[ModelTransform::default()]);

            let quads = vec![[FontVert::default(); 4]; Text::MAX_SYMBOLS];
            let data = MeshData::from_quads(&quads).expect("create atlas mesh");
            let mesh = context.create_mesh(&data);

            Text {
                map,
                size,
                atlas,
                instance,
                mesh,
                n: 0,
            }
        };

        Self {
            texture_layer,
            color_layer,
            font_layer,
            texture_globals,
            color_globals,
            lights,
            spaces,
            post,
            sprites,
            sprite_meshes,
            squares,
            text,
            camera: Camera::default(),
            time_passed: 0.,
            fullscreen: false,
        }
    }
}

impl Loop for App {
    fn update(&mut self, context: &mut Context, input: &Input) {
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

        self.time_passed += input.delta_time * LIGHTS_SPEED;
        let make_source = |step, col| {
            let step = (self.time_passed + step) % TAU;
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

        let sources = LIGHTS.map(|(step, col)| make_source(step, col));
        self.lights
            .update_sources(0, 0, &sources)
            .expect("update sources");

        let g = self.time_passed.sin() * 0.5 + 0.5;
        let col = Color([0.2, g, 0.5]);
        let colors = vec![ModelColor::from(col); self.squares.len];
        self.squares
            .color
            .update(&colors)
            .expect("update color instance");

        // Handle pressed keys
        #[cfg(not(target_arch = "wasm32"))]
        for key in input.pressed_keys {
            match key {
                Key::Escape => context.plan_to_close(),
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

        self.texture_globals.update_view(view);
        self.color_globals.update_view(view);

        self.sprite_meshes
            .iter()
            .filter(|sprite| sprite.update_view)
            .try_for_each(|sprite| {
                let transform = Transform {
                    pos: sprite.pos.into(),
                    rot: view.rotation().conjugate(),
                    ..Default::default()
                };

                sprite.instance.update(&[ModelTransform::from(transform)])
            })
            .expect("update instances");

        let backend = context.info().backend;
        let fps = context.fps();
        let s = format!("Backend: {backend:?} ({fps})");
        self.text.write(&s, context.size());
    }

    fn render(&self, frame: &mut Frame) {
        {
            let clear_color = Rgba::from_standard_bytes([46, 34, 47, 255]);
            let mut layer = frame
                .layer(&self.texture_layer)
                .with_clear_color(clear_color)
                .with_clear_depth()
                .start();

            layer
                .bind_globals(&self.texture_globals)
                .bind_textures(&self.sprites)
                .bind_lights(&self.lights)
                .bind_spaces(&self.spaces);

            for model in &self.sprite_meshes {
                layer.draw(model.mesh, &model.instance);
            }
        }

        {
            frame
                .layer(&self.color_layer)
                .start()
                .bind_globals(&self.color_globals)
                .bind_instance_color(&self.squares.color)
                .draw(&self.squares.mesh, &self.squares.instance);
        }

        frame.draw_on_screen_with(&self.post);

        {
            let clear_color = Rgba::from_bytes([0; 4]);
            frame
                .layer(&self.font_layer)
                .with_clear_color(clear_color)
                .start()
                .bind_textures(&self.text.map)
                .draw_limited(&self.text.mesh, &self.text.instance, self.text.n);
        }
    }
}
