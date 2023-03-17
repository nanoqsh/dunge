use {
    crate::{
        camera::{Camera, Projection, View},
        depth_frame::DepthFrame,
        frame::Frame,
        handles::*,
        instance::Instance,
        layout::InstanceModel,
        mesh::{Data as MeshData, Mesh},
        pipeline::Pipeline,
        pipeline::{Blend, PipelineParameters, Topology},
        r#loop::Loop,
        render_frame::{FrameFilter, RenderFrame},
        screen::Screen,
        shader::{self, Shader},
        shader_data::PostShaderData,
        storage::Storage,
        texture::{Data as TextureData, Texture},
        vertex::Vertex,
        Error,
    },
    once_cell::unsync::OnceCell,
    wgpu::{
        BindGroupLayout, Device, Queue, ShaderModule, Surface, SurfaceConfiguration, SurfaceError,
    },
    winit::window::Window,
};

pub(crate) struct Render {
    device: Device,
    queue: Queue,
    surface: Surface,
    config: SurfaceConfiguration,
    screen: Screen,
    shaders: Shaders,
    layouts: Layouts,
    post_pipeline: Pipeline,
    post_shader_data: PostShaderData,
    render_frame: RenderFrame,
    depth_frame: DepthFrame,
    resources: Resources,
}

impl Render {
    pub async fn new(window: &Window) -> Self {
        use wgpu::*;

        const TDIFF_BINDING: u32 = {
            assert!(shader::TEXTURED_TDIFF_BINDING == shader::FLAT_TDIFF_BINDING);
            shader::TEXTURED_TDIFF_BINDING
        };

        const SDIFF_BINDING: u32 = {
            assert!(shader::TEXTURED_SDIFF_BINDING == shader::FLAT_SDIFF_BINDING);
            shader::TEXTURED_SDIFF_BINDING
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

        let textured_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: TDIFF_BINDING,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: SDIFF_BINDING,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture bind group layout"),
        });

        let camera_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: shader::TEXTURED_CAMERA_BINDING,
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

        let post_shader_data_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: shader::POST_DATA_BINDING,
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

        let render_frame =
            RenderFrame::new((1, 1), FrameFilter::Nearest, &device, &textured_layout);
        let depth_frame = DepthFrame::new((1, 1), &device);

        let post_shader_data = PostShaderData::new(&device, &post_shader_data_layout);

        let shaders = Shaders::default();
        let layouts = Layouts {
            textured_layout,
            camera_layout,
            post_shader_data_layout,
        };

        let post_pipeline = Pipeline::new(
            &device,
            &shaders,
            &layouts,
            config.format,
            Shader::Post,
            PipelineParameters {
                blend: Blend::AlphaBlending,
                topology: Topology::TriangleStrip,
                cull_faces: false,
                ..Default::default()
            },
            false,
        );

        Self {
            device,
            queue,
            surface,
            config,
            screen: Screen::default(),
            shaders,
            layouts,
            post_pipeline,
            post_shader_data,
            render_frame,
            depth_frame,
            resources: Resources::default(),
        }
    }

    pub fn create_layer<V>(&mut self, params: PipelineParameters) -> LayerHandle<V>
    where
        V: Vertex,
    {
        let pipeline = Pipeline::new(
            &self.device,
            &self.shaders,
            &self.layouts,
            self.config.format,
            Shader::from_vertex_type::<V>(),
            params,
            true,
        );

        let id = self.resources.layers.insert(pipeline);
        LayerHandle::new(id)
    }

    pub fn delete_layer<V>(&mut self, handle: LayerHandle<V>) -> Result<(), Error> {
        self.resources.layers.remove(handle.id())
    }

    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        let texture = Texture::new(
            data,
            &self.device,
            &self.queue,
            &self.layouts.textured_layout,
        );

        let id = self.resources.textures.insert(texture);
        TextureHandle(id)
    }

    pub fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        self.resources
            .textures
            .get_mut(handle.0)
            .map(|texture| texture.update_data(data, &self.queue))
    }

    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        self.resources.textures.remove(handle.0)
    }

    pub fn create_instances(&mut self, models: &[InstanceModel]) -> InstanceHandle {
        let instance = Instance::new(models, &self.device);
        let id = self.resources.instances.insert(instance);
        InstanceHandle(id)
    }

    pub fn update_instances(
        &mut self,
        handle: InstanceHandle,
        models: &[InstanceModel],
    ) -> Result<(), Error> {
        self.resources
            .instances
            .get_mut(handle.0)
            .map(|instances| instances.update_models(models, &self.queue))
    }

    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        self.resources.instances.remove(handle.0)
    }

    pub fn create_mesh<V>(&mut self, data: &MeshData<V>) -> MeshHandle<V>
    where
        V: Vertex,
    {
        let mesh = Mesh::new(data, &self.device);
        let id = self.resources.meshes.insert(mesh);
        MeshHandle::new(id)
    }

    pub fn update_mesh<V>(&mut self, handle: MeshHandle<V>, data: &MeshData<V>) -> Result<(), Error>
    where
        V: Vertex,
    {
        self.resources
            .meshes
            .get_mut(handle.id())
            .map(|mesh| mesh.update_data(data, &self.device, &self.queue))
    }

    pub fn delete_mesh<V>(&mut self, handle: MeshHandle<V>) -> Result<(), Error> {
        self.resources.meshes.remove(handle.id())
    }

    pub fn create_view(&mut self, view: View<Projection>) -> ViewHandle {
        let mut camera = Camera::new(&self.device, &self.layouts.camera_layout);
        camera.set_view(view);
        let id = self.resources.views.insert(camera);
        ViewHandle(id)
    }

    pub fn update_view(&mut self, handle: ViewHandle, view: View<Projection>) -> Result<(), Error> {
        self.resources
            .views
            .get_mut(handle.0)
            .map(|camera| camera.set_view(view))
    }

    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        self.resources.views.remove(handle.0)
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn set_screen(&mut self, screen: Option<Screen>) {
        if let Some(screen) = screen {
            self.screen = screen;
        }

        self.config.width = self.screen.width.get();
        self.config.height = self.screen.height.get();
        self.surface.configure(&self.device, &self.config);

        let virt_size = self.screen.as_virtual_size();
        self.post_shader_data.resize(virt_size, &self.queue);

        self.render_frame = RenderFrame::new(
            virt_size,
            self.screen.filter,
            &self.device,
            &self.layouts.textured_layout,
        );

        self.depth_frame = DepthFrame::new(virt_size, &self.device);
    }

    pub fn draw_frame<L>(&mut self, lp: &L) -> RenderResult<L::Error>
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
        output.present();

        RenderResult::Ok
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn post_pipeline(&self) -> &Pipeline {
        &self.post_pipeline
    }

    pub fn post_shader_data(&self) -> &PostShaderData {
        &self.post_shader_data
    }

    pub fn render_frame(&self) -> &RenderFrame {
        &self.render_frame
    }

    pub fn depth_frame(&self) -> &DepthFrame {
        &self.depth_frame
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
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

pub(crate) enum RenderResult<E> {
    Ok,
    SurfaceError(SurfaceError),
    Error(E),
}

#[derive(Default)]
pub(crate) struct Shaders {
    color: OnceCell<ShaderModule>,
    flat: OnceCell<ShaderModule>,
    post: OnceCell<ShaderModule>,
    textured: OnceCell<ShaderModule>,
}

impl Shaders {
    pub fn module(&self, device: &Device, shader: Shader) -> &ShaderModule {
        use wgpu::{ShaderModuleDescriptor, ShaderSource};

        let cell = match shader {
            Shader::Color => &self.color,
            Shader::Flat => &self.flat,
            Shader::Post => &self.post,
            Shader::Textured => &self.textured,
        };

        cell.get_or_init(|| {
            device.create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(shader.source().into()),
            })
        })
    }
}

pub(crate) enum BindGroupLayouts<'a> {
    N1([&'a BindGroupLayout; 1]),
    N2([&'a BindGroupLayout; 2]),
}

impl<'a> BindGroupLayouts<'a> {
    pub fn as_slice(&self) -> &[&'a BindGroupLayout] {
        match self {
            Self::N1(b) => b,
            Self::N2(b) => b,
        }
    }
}

pub(crate) struct Layouts {
    textured_layout: BindGroupLayout,
    camera_layout: BindGroupLayout,
    post_shader_data_layout: BindGroupLayout,
}

impl Layouts {
    pub fn bind_group_layouts(&self, shader: Shader) -> BindGroupLayouts {
        match shader {
            Shader::Color => BindGroupLayouts::N1([&self.camera_layout]),
            Shader::Flat => BindGroupLayouts::N1([&self.textured_layout]),
            Shader::Post => {
                BindGroupLayouts::N2([&self.post_shader_data_layout, &self.textured_layout])
            }
            Shader::Textured => BindGroupLayouts::N2([&self.camera_layout, &self.textured_layout]),
        }
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) layers: Storage<Pipeline>,
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) views: Storage<Camera>,
}
