use {
    crate::{
        camera::{Camera, Projection, View},
        color::Linear,
        frame::{Frame, MainPipeline, Resources},
        instance::Instance,
        layout::{layout, InstanceModel},
        mesh::{Mesh, MeshData},
        pipline::{Pipeline, PipelineData},
        r#loop::Loop,
        screen::Screen,
        shader_consts,
        size::Size,
        texture::{DepthFrame, FrameFilter, RenderFrame, Texture, TextureData},
        vertex::{ColorVertex, TextureVertex, Vertex},
        Error,
    },
    wgpu::{
        BindGroupLayout, Color, Device, LoadOp, Queue, Surface, SurfaceConfiguration, SurfaceError,
    },
    winit::window::Window,
};

pub(crate) struct Render {
    device: Device,
    queue: Queue,
    main_pipeline: MainPipeline,
    post_pipeline: Pipeline,
    surface: Surface,
    config: SurfaceConfiguration,
    size: Size,
    texture_layout: BindGroupLayout,
    load: LoadOp<Color>,
    camera: Camera,
    screen: Screen,
    resources: Resources,
    render_frame: RenderFrame,
    depth_frame: DepthFrame,
}

impl Render {
    pub(crate) async fn new(window: &Window) -> Self {
        use wgpu::*;

        #[cfg(target_os = "android")]
        {
            wait_for_native_screen();
        }

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
                    limits: Limits {
                        max_storage_buffers_per_shader_stage: 1,
                        max_storage_textures_per_shader_stage: 1,
                        ..if cfg!(target_arch = "wasm32") {
                            Limits::downlevel_webgl2_defaults()
                        } else {
                            Limits::default()
                        }
                    },
                    label: None,
                },
                None,
            )
            .await
            .expect("request device");

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };

        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: shader_consts::textured::T_DIFFUSE.binding,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: shader_consts::textured::S_DIFFUSE.binding,
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

        let depth_stencil = DepthStencilState {
            format: DepthFrame::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        let textured_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/textured.wgsl"),
                bind_group_layouts: &[&camera_layout, &texture_layout],
                vertex_buffers: &[layout::<TextureVertex>(), layout::<InstanceModel>()],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                depth_stencil: Some(depth_stencil.clone()),
            };
            Pipeline::new(&device, data)
        };

        let color_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/color.wgsl"),
                bind_group_layouts: &[&camera_layout],
                vertex_buffers: &[layout::<ColorVertex>(), layout::<InstanceModel>()],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                depth_stencil: Some(depth_stencil),
            };
            Pipeline::new(&device, data)
        };

        let screen_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: shader_consts::post::SCREEN.binding,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("screen bind group layout"),
        });

        let post_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/post.wgsl"),
                bind_group_layouts: &[&screen_layout, &texture_layout],
                vertex_buffers: &[],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleStrip,
                cull_mode: None,
                depth_stencil: None,
            };
            Pipeline::new(&device, data)
        };

        let camera = Camera::new(&device, &camera_layout);

        let render_frame = RenderFrame::new((1, 1), FrameFilter::Nearest, &device, &texture_layout);
        let depth_frame = DepthFrame::new((1, 1), &device);

        let screen = Screen::new(&device, &screen_layout);

        Self {
            device,
            queue,
            main_pipeline: MainPipeline {
                textured: textured_pipeline,
                color: color_pipeline,
            },
            post_pipeline,
            surface,
            config,
            size: Size::default(),
            texture_layout,
            load: LoadOp::Load,
            camera,
            resources: Resources::default(),
            render_frame,
            depth_frame,
            screen,
        }
    }

    pub(crate) fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        let texture = Texture::new(data, &self.device, &self.queue, &self.texture_layout);
        let id = self.resources.textures.insert(texture);
        TextureHandle(id)
    }

    pub(crate) fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        self.resources
            .textures
            .get_mut(handle.0)
            .map(|texture| texture.update_data(data, &self.queue))
    }

    pub(crate) fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.resources.textures.remove(handle.0)
    }

    pub(crate) fn create_instances(&mut self, models: &[InstanceModel]) -> InstanceHandle {
        let instance = Instance::new(models, &self.device);
        let id = self.resources.instances.insert(instance);
        InstanceHandle(id)
    }

    pub(crate) fn update_instances(
        &mut self,
        handle: InstanceHandle,
        models: &[InstanceModel],
    ) -> Result<(), Error> {
        self.resources
            .instances
            .get_mut(handle.0)
            .map(|instances| instances.update_models(models, &self.queue))
    }

    pub(crate) fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        self.resources.instances.remove(handle.0)
    }

    pub(crate) fn create_mesh<V>(&mut self, data: MeshData<V>) -> MeshHandle
    where
        V: Vertex,
    {
        let mesh = Mesh::new(data, &self.device);
        let id = self.resources.meshes.insert(mesh);
        MeshHandle(id)
    }

    pub(crate) fn update_mesh<V>(
        &mut self,
        handle: MeshHandle,
        data: MeshData<V>,
    ) -> Result<(), Error>
    where
        V: Vertex,
    {
        self.resources
            .meshes
            .get_mut(handle.0)
            .map(|meshe| meshe.update_data(data, &self.queue))
    }

    pub(crate) fn delete_mesh(&mut self, handle: MeshHandle) -> Result<(), Error> {
        self.resources.meshes.remove(handle.0)
    }

    pub(crate) fn set_clear_color(&mut self, col: Option<Linear<f64>>) {
        self.load = col
            .map(|Linear([r, g, b, a])| LoadOp::Clear(Color { r, g, b, a }))
            .unwrap_or(LoadOp::Load);
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.camera.set_view(view);
        self.camera.resize(self.size.as_virtual(), &self.queue);
    }

    pub(crate) fn size(&self) -> Size {
        self.size
    }

    pub(crate) fn resize(&mut self, size: Option<Size>) {
        if let Some(size) = size {
            self.size = size;
        }

        self.config.width = self.size.width.get();
        self.config.height = self.size.height.get();
        self.surface.configure(&self.device, &self.config);

        let virt = self.size.as_virtual();
        self.camera.resize(virt, &self.queue);
        self.screen.resize(virt, &self.queue);

        self.render_frame =
            RenderFrame::new(virt, self.size.filter, &self.device, &self.texture_layout);
        self.depth_frame = DepthFrame::new(virt, &self.device);
    }

    pub(crate) fn draw_frame<L>(&mut self, lp: &L) -> RenderResult<L::Error>
    where
        L: Loop,
    {
        use wgpu::*;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // Main render pass
        {
            let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("textured render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.render_frame.view(),
                    resolve_target: None,
                    ops: Operations {
                        load: self.load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_frame.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            let mut frame = Frame::new(
                &self.main_pipeline,
                self.camera.bind_group(),
                &self.resources,
                pass,
            );

            if let Err(err) = lp.render(&mut frame) {
                return RenderResult::Error(err);
            }
        }

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(err) => return RenderResult::SurfaceError(err),
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Post render pass
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("post render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            pass.set_pipeline(self.post_pipeline.as_ref());
            pass.set_bind_group(
                shader_consts::post::T_DIFFUSE.group,
                self.render_frame.bind_group(),
                &[],
            );
            pass.set_bind_group(
                shader_consts::post::SCREEN.group,
                self.screen.bind_group(),
                &[],
            );

            pass.draw(0..4, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        output.present();

        RenderResult::Ok
    }
}

#[cfg(target_os = "android")]
fn wait_for_native_screen() {
    log::info!("waiting for native screen");
    loop {
        if let Some(window) = ndk_glue::native_window().as_ref() {
            log::info!("native screen found:{:?}", window);
            break;
        }
    }
}

pub(crate) enum RenderResult<E> {
    Ok,
    SurfaceError(SurfaceError),
    Error(E),
}

/// A texture handle.
#[derive(Clone, Copy)]
pub struct TextureHandle(pub(crate) u32);

/// A mesh handle.
#[derive(Clone, Copy)]
pub struct MeshHandle(pub(crate) u32);

/// An instance handle.
#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);
