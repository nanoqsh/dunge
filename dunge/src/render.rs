use {
    crate::{
        camera::{Camera, Projection, View},
        color::Linear,
        frame::{Frame, Resources},
        instance::Instance,
        layout::{layout, ColorVertex, InstanceModel, Layout, TextureVertex},
        mesh::{Mesh, MeshData},
        pipline::{Pipeline, PipelineData},
        r#loop::Loop,
        size::Size,
        texture::{DepthFrame, FrameFilter, RenderFrame, Texture, TextureData},
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
    textured_pipeline: Pipeline,
    _color_pipeline: Pipeline,
    post_pipeline: Pipeline,
    surface: Surface,
    config: SurfaceConfiguration,
    size: Size,
    texture_layout: BindGroupLayout,
    load: LoadOp<Color>,
    camera: Camera,
    resources: Resources,
    render_frame: RenderFrame,
    depth_frame: DepthFrame,
}

impl Render {
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const TEXTURE_GROUP: u32 = 1;
    pub(crate) const TEXTURE_GROUP_IN_POST: u32 = 0;

    pub(crate) const TEXTURE_BINDING: u32 = 0;
    pub(crate) const TEXTURE_SAMPLER_BINDING: u32 = 1;

    pub(crate) const VERTEX_BUFFER_SLOT: u32 = 0;
    pub(crate) const INSTANCE_BUFFER_SLOT: u32 = 1;

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
            format: TextureFormat::Rgba8UnormSrgb,
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };

        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: Self::TEXTURE_BINDING,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: Self::TEXTURE_SAMPLER_BINDING,
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

        let post_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/post.wgsl"),
                bind_group_layouts: &[&texture_layout],
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

        Self {
            device,
            queue,
            textured_pipeline,
            _color_pipeline: color_pipeline,
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
        V: Layout,
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
        V: Layout,
    {
        self.resources
            .meshes
            .get_mut(handle.0)
            .map(|meshe| meshe.update_data(data, &self.queue))
    }

    pub(crate) fn delete_mesh(&mut self, handle: MeshHandle) -> Result<(), Error> {
        self.resources.meshes.remove(handle.0)
    }

    pub(crate) fn set_clear_color(&mut self, Linear([r, g, b, a]): Linear<f64>) {
        self.load = LoadOp::Clear(Color { r, g, b, a });
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.camera.set_view(view);
        self.camera.resize(self.size.as_physical(), &self.queue);
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

        self.camera.resize(self.size.as_physical(), &self.queue);

        let virt = self.size.as_virtual();
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
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
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

            pass.set_pipeline(self.textured_pipeline.as_ref());
            pass.set_bind_group(Self::CAMERA_GROUP, self.camera.bind_group(), &[]);

            let mut frame = Frame::new(&self.resources, pass);
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
                Self::TEXTURE_GROUP_IN_POST,
                self.render_frame.bind_group(),
                &[],
            );
            pass.draw(0..4, 0..1);
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

#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);
