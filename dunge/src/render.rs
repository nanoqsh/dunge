#![allow(clippy::wildcard_imports)]

use {
    crate::{
        camera::{Camera, Projection, View},
        frame::Frame,
        instance::Instance,
        layer::Resources,
        layout::{layout, InstanceModel},
        mesh::{Data as MeshData, Mesh},
        pipline::{Pipeline, PipelineData},
        r#loop::Loop,
        shader_consts,
        shader_data::PostShaderData,
        size::Size,
        texture::{Data as TextureData, DepthFrame, FrameFilter, RenderFrame, Texture},
        vertex::{ColorVertex, FlatVertex, TextureVertex, Vertex},
        Error,
    },
    std::marker::PhantomData,
    wgpu::{BindGroupLayout, Device, Queue, Surface, SurfaceConfiguration, SurfaceError},
    winit::window::Window,
};

pub(crate) struct Render {
    device: Device,
    queue: Queue,
    texture_pipeline: Pipeline,
    color_pipeline: Pipeline,
    flat_pipeline: Pipeline,
    post_pipeline: Pipeline,
    surface: Surface,
    config: SurfaceConfiguration,
    size: Size,
    texture_layout: BindGroupLayout,
    camera_layout: BindGroupLayout,
    post_shader_data: PostShaderData,
    resources: Resources,
    render_frame: RenderFrame,
    depth_frame: DepthFrame,
}

impl Render {
    pub(crate) async fn new(window: &Window) -> Self {
        use wgpu::*;

        const _: () = {
            assert!(
                shader_consts::textured::T_DIFFUSE.binding
                    == shader_consts::flat::T_DIFFUSE.binding
            );

            assert!(
                shader_consts::textured::S_DIFFUSE.binding
                    == shader_consts::flat::S_DIFFUSE.binding
            );
        };

        #[cfg(target_os = "android")]
        {
            Self::wait_for_native_screen();
        }

        let instance = Instance::default();
        let surface = unsafe { instance.create_surface(window).expect("create surface") };
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
            view_formats: vec![],
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
                binding: shader_consts::textured::CAMERA.binding,
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

        let texture_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/textured.wgsl"),
                bind_group_layouts: &[&camera_layout, &texture_layout],
                vertex_buffers: &[layout::<TextureVertex>(), layout::<InstanceModel>()],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                depth_stencil: Some(DepthStencilState {
                    format: DepthFrame::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
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
                depth_stencil: Some(DepthStencilState {
                    format: DepthFrame::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
            };
            Pipeline::new(&device, data)
        };

        let flat_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/flat.wgsl"),
                bind_group_layouts: &[&texture_layout],
                vertex_buffers: &[layout::<FlatVertex>(), layout::<InstanceModel>()],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                depth_stencil: Some(DepthStencilState {
                    format: DepthFrame::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: CompareFunction::Always,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
            };
            Pipeline::new(&device, data)
        };

        let post_shader_data_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: shader_consts::post::DATA.binding,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("post shader data bind group layout"),
        });

        let post_pipeline = {
            let data = PipelineData {
                shader_src: include_str!("shaders/post.wgsl"),
                bind_group_layouts: &[&post_shader_data_layout, &texture_layout],
                vertex_buffers: &[],
                fragment_texture_format: config.format,
                topology: PrimitiveTopology::TriangleStrip,
                cull_mode: None,
                depth_stencil: None,
            };
            Pipeline::new(&device, data)
        };

        let render_frame = RenderFrame::new((1, 1), FrameFilter::Nearest, &device, &texture_layout);
        let depth_frame = DepthFrame::new((1, 1), &device);

        let post_shader_data = PostShaderData::new(&device, &post_shader_data_layout);

        Self {
            device,
            queue,
            texture_pipeline,
            color_pipeline,
            flat_pipeline,
            post_pipeline,
            surface,
            config,
            size: Size::default(),
            texture_layout,
            camera_layout,
            resources: Resources::default(),
            render_frame,
            depth_frame,
            post_shader_data,
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

    pub(crate) fn create_mesh<V>(&mut self, data: &MeshData<V>) -> MeshHandle<V>
    where
        V: Vertex,
    {
        let mesh = Mesh::new(data, &self.device);
        let id = self.resources.meshes.insert(mesh);
        MeshHandle::new(id)
    }

    pub(crate) fn update_mesh<V>(
        &mut self,
        handle: MeshHandle<V>,
        data: &MeshData<V>,
    ) -> Result<(), Error>
    where
        V: Vertex,
    {
        self.resources
            .meshes
            .get_mut(handle.id())
            .map(|mesh| mesh.update_data(data, &self.queue))
    }

    pub(crate) fn delete_mesh<V>(&mut self, handle: MeshHandle<V>) -> Result<(), Error> {
        self.resources.meshes.remove(handle.id())
    }

    pub(crate) fn create_view(&mut self, view: View<Projection>) -> ViewHandle {
        let mut camera = Camera::new(&self.device, &self.camera_layout);
        camera.set_view(view);
        let id = self.resources.views.insert(camera);
        ViewHandle(id)
    }

    pub(crate) fn update_view(
        &mut self,
        handle: ViewHandle,
        view: View<Projection>,
    ) -> Result<(), Error> {
        self.resources
            .views
            .get_mut(handle.0)
            .map(|camera| camera.set_view(view))
    }

    pub(crate) fn delete_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        self.resources.views.remove(handle.0)
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
        self.post_shader_data.resize(virt, &self.queue);

        self.render_frame =
            RenderFrame::new(virt, self.size.filter, &self.device, &self.texture_layout);
        self.depth_frame = DepthFrame::new(virt, &self.device);
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

        let frame_view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut frame = Frame::new(self, frame_view);
        if let Err(err) = lp.render(&mut frame) {
            return RenderResult::Error(err);
        }

        frame.commit_in_frame();
        frame.submit();
        output.present();

        RenderResult::Ok
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn post_pipeline(&self) -> &Pipeline {
        &self.post_pipeline
    }

    pub(crate) fn post_shader_data(&self) -> &PostShaderData {
        &self.post_shader_data
    }

    pub(crate) fn resources(&self) -> &Resources {
        &self.resources
    }

    pub(crate) fn render_frame(&self) -> &RenderFrame {
        &self.render_frame
    }

    pub(crate) fn depth_frame(&self) -> &DepthFrame {
        &self.depth_frame
    }

    #[cfg(target_os = "android")]
    fn wait_for_native_screen() {
        loop {
            log::info!("waiting for native screen");
            if let Some(window) = ndk_glue::native_window().as_ref() {
                log::info!("native screen found:{:?}", window);
                break;
            }
        }
    }
}

pub(crate) trait GetPipeline<V> {
    fn get_pipeline(&self) -> &Pipeline;
}

impl GetPipeline<TextureVertex> for Render {
    fn get_pipeline(&self) -> &Pipeline {
        &self.texture_pipeline
    }
}

impl GetPipeline<ColorVertex> for Render {
    fn get_pipeline(&self) -> &Pipeline {
        &self.color_pipeline
    }
}

impl GetPipeline<FlatVertex> for Render {
    fn get_pipeline(&self) -> &Pipeline {
        &self.flat_pipeline
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
pub struct MeshHandle<V>(u32, PhantomData<V>);

impl<V> MeshHandle<V> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

/// An instance handle.
#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);

/// A view handle.
#[derive(Clone, Copy)]
pub struct ViewHandle(pub(crate) u32);
