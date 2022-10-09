use {
    crate::{
        camera::{Camera, Projection, View},
        color::Linear,
        frame::{Frame, Resources},
        mesh::{Mesh, MeshData},
        r#loop::Loop,
        size::Size,
        texture::{Depth, Texture, TextureData},
        vertex::{layout, TextureVertex, Vertex},
    },
    std::num::NonZeroU32,
    wgpu::{
        BindGroupLayout, Color, Device, LoadOp, Queue, RenderPipeline, Surface,
        SurfaceConfiguration, SurfaceError,
    },
    winit::window::Window,
};

pub(crate) struct Render {
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    surface: Surface,
    config: SurfaceConfiguration,
    size: Size,
    texture_layout: BindGroupLayout,
    load: LoadOp<Color>,
    depth: Depth,
    camera: Camera,
    resources: Resources,
}

impl Render {
    pub(crate) const TEXTURE_BIND_GROUP: u32 = 0;
    pub(crate) const TEXTURE_SAMPLER_BIND_GROUP: u32 = 1;
    pub(crate) const CAMERA_BIND_GROUP: u32 = 1;
    pub(crate) const VERTEX_BUFFER_SLOT: u32 = 0;

    pub(crate) async fn new(window: &Window) -> Self {
        use wgpu::*;

        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: if cfg!(target_arch = "wasm32") {
                    PowerPreference::LowPower
                } else {
                    PowerPreference::HighPerformance
                },
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .expect("request device");

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };

        let shader = device.create_shader_module(include_wgsl!("shaders/textured.wgsl"));
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: Self::TEXTURE_BIND_GROUP,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: Self::TEXTURE_SAMPLER_BIND_GROUP,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture bind group layout"),
        });

        let camera_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera bind group layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("render pipeline layout"),
            bind_group_layouts: &[&texture_layout, &camera_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[layout::<TextureVertex>()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: Depth::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let camera = Camera::new(&device, &camera_layout);

        let depth = Depth::new(&device, &config);

        Self {
            device,
            queue,
            pipeline,
            surface,
            config,
            size: {
                let n = NonZeroU32::new(1).expect("1 is non zero");
                (n, n)
            },
            texture_layout,
            load: LoadOp::Load,
            depth,
            camera,
            resources: Resources::default(),
        }
    }

    pub(crate) fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        let texture = Texture::new(data, &self.device, &self.queue, &self.texture_layout);
        let id = self.resources.textures.insert(texture);
        TextureHandle(id)
    }

    pub(crate) fn delete_texture(&mut self, TextureHandle(id): TextureHandle) {
        self.resources.textures.remove(id);
    }

    pub(crate) fn create_mesh<V>(&mut self, data: MeshData<V>) -> MeshHandle
    where
        V: Vertex,
    {
        let mesh = Mesh::new(data, &self.device);
        let id = self.resources.meshes.insert(mesh);
        MeshHandle(id)
    }

    pub(crate) fn delete_mesh(&mut self, MeshHandle(id): MeshHandle) {
        self.resources.meshes.remove(id);
    }

    pub(crate) fn set_clear_color(&mut self, Linear([r, g, b, a]): Linear<f64>) {
        self.load = LoadOp::Clear(Color { r, g, b, a });
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.camera.set_view(view);
        self.camera.resize(self.size(), &self.queue);
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn resize(&mut self, size @ (width, height): Size) {
        self.size = size;
        self.config.width = width.get();
        self.config.height = height.get();
        self.surface.configure(&self.device, &self.config);
        self.depth = Depth::new(&self.device, &self.config);

        self.camera.resize(size, &self.queue);
    }

    pub(crate) fn draw_frame<L>(&mut self, lp: &L) -> RenderResult<L::Error>
    where
        L: Loop,
    {
        use wgpu::*;

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(err) => return RenderResult::SurfaceError(err),
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: self.load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(Self::CAMERA_BIND_GROUP, self.camera.bind_group(), &[]);

            let mut frame = Frame {
                pass,
                resources: &self.resources,
            };

            if let Err(err) = lp.render(&mut frame) {
                return RenderResult::Error(err);
            }
        }

        self.queue.submit([encoder.finish()]);
        output.present();

        RenderResult::Ok
    }
}

pub(crate) enum RenderResult<E> {
    Ok,
    SurfaceError(SurfaceError),
    Error(E),
}

#[derive(Clone, Copy)]
pub struct TextureHandle(pub(crate) u32);

#[derive(Clone, Copy)]
pub struct MeshHandle(pub(crate) u32);
