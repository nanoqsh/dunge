use {
    crate::{
        camera::{Camera, Projection, View},
        context::Screenshot,
        depth_frame::DepthFrame,
        frame::Frame,
        handles::*,
        instance::{Instance, InstanceModel},
        mesh::{Data as MeshData, Mesh},
        pipeline::{Pipeline, PipelineParameters},
        r#loop::Loop,
        render_frame::{FrameFilter, RenderFrame},
        screen::Screen,
        shader::{self, Shader},
        shader_data::{Ambient, Light, PostShaderData, SourceModel},
        storage::Storage,
        texture::{Data as TextureData, Texture},
        topology::Topology,
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
    ambient: Ambient,
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

        let mut adapter = None;
        for ad in instance.enumerate_adapters(Backends::all()) {
            let info = ad.get_info();
            if info.backend == Backend::Gl {
                adapter = Some(ad);
                break;
            }
        }

        let adapter = adapter.expect("gl adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits {
                        max_storage_buffers_per_shader_stage: 0,
                        max_storage_textures_per_shader_stage: 0,
                        max_dynamic_storage_buffers_per_pipeline_layout: 0,
                        max_storage_buffer_binding_size: 0,
                        max_compute_workgroup_storage_size: 0,
                        max_compute_invocations_per_workgroup: 0,
                        max_compute_workgroup_size_x: 0,
                        max_compute_workgroup_size_y: 0,
                        max_compute_workgroup_size_z: 0,
                        max_compute_workgroups_per_dimension: 0,
                        ..if cfg!(target_arch = "wasm32") {
                            Limits::downlevel_webgl2_defaults()
                        } else {
                            Limits::downlevel_defaults()
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
            format: RenderFrame::RENDER_FORMAT,
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
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("post shader data bind group layout"),
        });

        let lights_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: shader::TEXTURED_SOURCES_BINDING,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: shader::TEXTURED_N_SOURCES_BINDING,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("lights bind group layout"),
        });

        let ambient_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: shader::TEXTURED_AMBIENT_BINDING,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("ambient bind group layout"),
        });

        let render_frame =
            RenderFrame::new((1, 1), FrameFilter::Nearest, &device, &textured_layout);
        let depth_frame = DepthFrame::new((1, 1), &device);

        let post_shader_data = PostShaderData::new(&device, &post_shader_data_layout);
        let ambient = Ambient::new(&device, &ambient_layout);
        let shaders = Shaders::default();

        let mut resources = Resources::default();
        resources
            .lights
            .insert(Light::new(&[], &device, &lights_layout).expect("default light"));

        // TODO: Can I create all layouts right in `Layouts`?
        let layouts = Layouts {
            textured_layout,
            camera_layout,
            post_shader_data_layout,
            lights_layout,
            ambient_layout,
        };

        let post_pipeline = Pipeline::new(
            &device,
            &shaders,
            &layouts,
            config.format,
            Shader::Post,
            PipelineParameters {
                blend: BlendState::ALPHA_BLENDING,
                topology: PrimitiveTopology::TriangleStrip,
                cull_faces: false,
                depth_stencil: None,
                ..Default::default()
            },
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
            ambient,
            render_frame,
            depth_frame,
            resources,
        }
    }

    pub fn create_layer<V, T>(&mut self, params: PipelineParameters) -> LayerHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        let pipeline = Pipeline::new(
            &self.device,
            &self.shaders,
            &self.layouts,
            self.config.format,
            V::VALUE.into_inner(),
            PipelineParameters {
                topology: T::VALUE.into_inner(),
                ..params
            },
        );

        let id = self.resources.layers.insert(pipeline);
        LayerHandle::new(id)
    }

    pub fn delete_layer<V, T>(&mut self, handle: LayerHandle<V, T>) -> Result<(), Error> {
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

    pub fn create_mesh<V, T>(&mut self, data: &MeshData<V, T>) -> MeshHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        let mesh = Mesh::new(data, &self.device);
        let id = self.resources.meshes.insert(mesh);
        MeshHandle::new(id)
    }

    pub fn update_mesh<V, T>(
        &mut self,
        handle: MeshHandle<V, T>,
        data: &MeshData<V, T>,
    ) -> Result<(), Error>
    where
        V: Vertex,
        T: Topology,
    {
        self.resources
            .meshes
            .get_mut(handle.id())
            .map(|mesh| mesh.update_data(data, &self.device, &self.queue))
    }

    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), Error> {
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

    pub fn create_light(&mut self, srcs: &[SourceModel]) -> Result<LightHandle, Error> {
        let light = Light::new(srcs, &self.device, &self.layouts.lights_layout)?;
        let id = self.resources.lights.insert(light);
        Ok(LightHandle(id))
    }

    pub fn update_light(&mut self, handle: LightHandle, srcs: &[SourceModel]) -> Result<(), Error> {
        let light = self.resources.lights.get_mut(handle.0)?;
        if !light.update(srcs, &self.queue) {
            *light = Light::new(srcs, &self.device, &self.layouts.lights_layout)?;
        }

        Ok(())
    }

    pub fn update_nth_light(
        &mut self,
        handle: LightHandle,
        n: usize,
        source: SourceModel,
    ) -> Result<(), Error> {
        self.resources
            .lights
            .get_mut(handle.0)
            .and_then(|light| light.update_nth(n, source, &self.queue))
    }

    pub fn delete_light(&mut self, handle: LightHandle) -> Result<(), Error> {
        self.resources.lights.remove(handle.0)
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn set_screen(&mut self, screen: Option<Screen>) {
        if let Some(screen) = screen {
            self.screen = screen;
        }

        let (width, height) = self.screen.physical_size();
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let (vw, vh) = {
            let (vw, vh) = self.screen.virtual_size();
            (vw as f32, vh as f32)
        };

        let (aw, ah) = self.screen.virtual_size_aligned();
        let factor = [vw / aw as f32, vh / ah as f32];
        self.post_shader_data.resize([vw, vh], factor, &self.queue);

        self.render_frame = RenderFrame::new(
            (aw, ah),
            self.screen.filter,
            &self.device,
            &self.layouts.textured_layout,
        );

        self.depth_frame = DepthFrame::new((aw, ah), &self.device);
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

    pub fn take_screenshot(&self) -> Screenshot {
        use {
            std::{num::NonZeroU32, sync::mpsc},
            wgpu::*,
        };

        const N_COLOR_CHANNELS: usize = 4;

        let image = ImageCopyTexture {
            texture: self.render_frame.texture(),
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        };

        let (width, height) = self.screen.virtual_size_aligned();
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("copy buffer"),
            size: width as u64 * height as u64 * N_COLOR_CHANNELS as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buffer = ImageCopyBuffer {
            buffer: &buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(width * N_COLOR_CHANNELS as u32),
                rows_per_image: NonZeroU32::new(height),
            },
        };

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(image, buffer, size);
        self.queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel();
        let buffer_slice = buffer.buffer.slice(..);
        buffer_slice.map_async(MapMode::Read, move |res| _ = sender.send(res));

        self.device.poll(Maintain::Wait);
        if receiver
            .recv()
            .expect("wait until the buffer maps")
            .is_err()
        {
            return Screenshot {
                width,
                height,
                data: vec![],
            };
        }

        let (vw, vh) = self.screen.virtual_size();
        let data = {
            let view = buffer_slice.get_mapped_range();
            let mut data = Vec::with_capacity(vw as usize * vh as usize * N_COLOR_CHANNELS);
            let row_size = width as usize * N_COLOR_CHANNELS;
            let virt_row_size = vw as usize * N_COLOR_CHANNELS;
            for row in view.chunks(row_size) {
                data.extend_from_slice(&row[..virt_row_size]);
            }

            if RenderFrame::RENDER_FORMAT == TextureFormat::Bgra8UnormSrgb {
                for chunk in data.chunks_mut(N_COLOR_CHANNELS) {
                    chunk.swap(0, 2);
                }
            }

            data
        };

        Screenshot {
            width: vw,
            height: vh,
            data,
        }
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

    pub fn ambient(&self) -> &Ambient {
        &self.ambient
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

pub(crate) struct Layouts {
    textured_layout: BindGroupLayout,
    camera_layout: BindGroupLayout,
    post_shader_data_layout: BindGroupLayout,
    lights_layout: BindGroupLayout,
    ambient_layout: BindGroupLayout,
}

impl Layouts {
    pub fn bind_group_layouts(&self, shader: Shader) -> BindGroupLayouts {
        match shader {
            Shader::Color => BindGroupLayouts::N1([&self.camera_layout]),
            Shader::Flat => BindGroupLayouts::N1([&self.textured_layout]),
            Shader::Post => {
                BindGroupLayouts::N2([&self.post_shader_data_layout, &self.textured_layout])
            }
            Shader::Textured => BindGroupLayouts::N4([
                &self.camera_layout,
                &self.textured_layout,
                &self.lights_layout,
                &self.ambient_layout,
            ]),
        }
    }
}

pub(crate) enum BindGroupLayouts<'a> {
    N1([&'a BindGroupLayout; 1]),
    N2([&'a BindGroupLayout; 2]),
    N4([&'a BindGroupLayout; 4]),
}

impl<'a> BindGroupLayouts<'a> {
    pub fn as_slice(&self) -> &[&'a BindGroupLayout] {
        match self {
            Self::N1(b) => b,
            Self::N2(b) => b,
            Self::N4(b) => b,
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
    pub(crate) lights: Storage<Light>,
}
