use {
    crate::{
        bind_groups::Layouts,
        camera::{Camera, Projection, View},
        color::Linear,
        context::Screenshot,
        error::{Error, ResourceNotFound, TooManySources, TooManySpaces},
        frame::Frame,
        framebuffer::Framebuffer,
        handles::*,
        instance::{Instance, InstanceModel},
        mesh::{Data as MeshData, Mesh},
        pipeline::{Pipeline, PipelineParameters},
        r#loop::Loop,
        screen::Screen,
        shader::Shader,
        shader_data::{
            Light, LightSpace, PostShaderData, SourceModel, SpaceData, SpaceModel, Texture,
            TextureData,
        },
        storage::Storage,
        topology::Topology,
        vertex::Vertex,
    },
    once_cell::unsync::OnceCell,
    std::num::NonZeroU32,
    wgpu::{Device, Queue, ShaderModule, Surface, SurfaceConfiguration, SurfaceError},
    winit::window::Window,
};

pub(crate) struct Render {
    device: Device,
    queue: Queue,
    surface: Surface,
    config: SurfaceConfiguration,
    screen: Screen,
    max_texture_size: u32,
    shaders: Shaders,
    layouts: Layouts,
    post_pipeline: Pipeline,
    post_shader_data: PostShaderData,
    framebuffer: Framebuffer,
    resources: Resources,
}

impl Render {
    pub async fn new(window: &Window) -> Self {
        use wgpu::*;

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

        let adapter = adapter.expect("selected adapter");
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

        let max_texture_size = device.limits().max_texture_dimension_2d;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: Framebuffer::RENDER_FORMAT,
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        let layouts = Layouts::new(&device);
        let post_shader_data = PostShaderData::new(&device, &layouts.post_shader_data);
        let shaders = Shaders::default();

        let mut resources = Resources::default();

        resources.lights.insert({
            const DEFAULT_AMBIENT: [f32; 3] = [1.; 3];

            Light::new(DEFAULT_AMBIENT, &[], &device, &layouts.lights).expect("default light")
        });

        resources.spaces.insert(
            LightSpace::new(&[], &[], &device, &queue, &layouts.space)
                .expect("default light space"),
        );

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

        let framebuffer = Framebuffer::new_default(&device, &layouts.textured);

        Self {
            device,
            queue,
            surface,
            config,
            screen: Screen::default(),
            max_texture_size,
            shaders,
            layouts,
            post_pipeline,
            post_shader_data,
            framebuffer,
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

    pub fn delete_layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<(), ResourceNotFound> {
        self.resources.layers.remove(handle.id())
    }

    pub fn create_texture(&mut self, data: TextureData) -> TextureHandle {
        let texture = Texture::new(data, &self.device, &self.queue, &self.layouts.textured);
        let id = self.resources.textures.insert(texture);
        TextureHandle(id)
    }

    pub fn update_texture(
        &mut self,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), ResourceNotFound> {
        self.resources
            .textures
            .get_mut(handle.0)
            .map(|texture| texture.update_data(data, &self.queue))
    }

    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), ResourceNotFound> {
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
    ) -> Result<(), ResourceNotFound> {
        self.resources
            .instances
            .get_mut(handle.0)
            .map(|instances| instances.update_models(models, &self.queue))
    }

    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), ResourceNotFound> {
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
    ) -> Result<(), ResourceNotFound>
    where
        V: Vertex,
        T: Topology,
    {
        self.resources
            .meshes
            .get_mut(handle.id())
            .map(|mesh| mesh.update_data(data, &self.device, &self.queue))
    }

    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), ResourceNotFound> {
        self.resources.meshes.remove(handle.id())
    }

    pub fn create_view(&mut self, view: View<Projection>) -> ViewHandle {
        let mut camera = Camera::new(&self.device, &self.layouts.camera);
        camera.set_view(view);
        let id = self.resources.views.insert(camera);
        ViewHandle(id)
    }

    pub fn update_view(
        &mut self,
        handle: ViewHandle,
        view: View<Projection>,
    ) -> Result<(), ResourceNotFound> {
        self.resources
            .views
            .get_mut(handle.0)
            .map(|camera| camera.set_view(view))
    }

    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<(), ResourceNotFound> {
        self.resources.views.remove(handle.0)
    }

    pub fn create_light(
        &mut self,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<LightHandle, TooManySources> {
        let light = Light::new(ambient.0, srcs, &self.device, &self.layouts.lights)?;
        let id = self.resources.lights.insert(light);
        Ok(LightHandle(id))
    }

    pub fn update_ambient(
        &mut self,
        handle: LightHandle,
        ambient: Linear<f32, 3>,
    ) -> Result<(), ResourceNotFound> {
        let light = self.resources.lights.get(handle.0)?;
        light.update_ambient(ambient.0, &self.queue);

        Ok(())
    }

    pub fn update_light(
        &mut self,
        handle: LightHandle,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<(), Error> {
        let light = self.resources.lights.get_mut(handle.0)?;
        if light.update_sources(srcs, &self.queue) {
            light.update_ambient(ambient.0, &self.queue);
        } else {
            *light = Light::new(ambient.0, srcs, &self.device, &self.layouts.lights)?;
        }

        Ok(())
    }

    pub fn update_nth_light(
        &mut self,
        handle: LightHandle,
        n: usize,
        source: SourceModel,
    ) -> Result<(), Error> {
        let light = self.resources.lights.get_mut(handle.0)?;
        light.update_nth(n, source, &self.queue)?;

        Ok(())
    }

    pub fn delete_light(&mut self, handle: LightHandle) -> Result<(), ResourceNotFound> {
        self.resources.lights.remove(handle.0)
    }

    pub fn create_space(
        &mut self,
        spaces: &[SpaceModel],
        data: &[SpaceData],
    ) -> Result<SpaceHandle, TooManySpaces> {
        let ls = LightSpace::new(spaces, data, &self.device, &self.queue, &self.layouts.space)?;
        let id = self.resources.spaces.insert(ls);
        Ok(SpaceHandle(id))
    }

    pub fn update_space(
        &mut self,
        handle: SpaceHandle,
        spaces: &[SpaceModel],
        data: &[SpaceData],
    ) -> Result<(), Error> {
        let ls = self.resources.spaces.get_mut(handle.0)?;
        if !ls.update_spaces(spaces, data, &self.queue) {
            *ls = LightSpace::new(spaces, data, &self.device, &self.queue, &self.layouts.space)?;
        }

        Ok(())
    }

    pub fn update_nth_space(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        space: SpaceModel,
    ) -> Result<(), Error> {
        let ls = self.resources.spaces.get_mut(handle.0)?;
        ls.update_nth_space(n, space, &self.queue)?;

        Ok(())
    }

    pub fn update_nth_space_color(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        color: Linear<f32, 3>,
    ) -> Result<(), Error> {
        let ls = self.resources.spaces.get_mut(handle.0)?;
        ls.update_nth_color(n, color.0, &self.queue)?;

        Ok(())
    }

    pub fn update_nth_space_data(
        &mut self,
        handle: SpaceHandle,
        n: usize,
        data: SpaceData,
    ) -> Result<(), Error> {
        let ls = self.resources.spaces.get_mut(handle.0)?;
        ls.update_nth_data(n, data, &self.queue)?;

        Ok(())
    }

    pub fn delete_space(&mut self, handle: SpaceHandle) -> Result<(), ResourceNotFound> {
        self.resources.spaces.remove(handle.0)
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

        let (bw, bh) = self.screen.buffer_size(self.max_texture_size);
        let (fw, fh) = self.screen.size_factor();
        self.post_shader_data
            .resize([bw.get() as f32, bh.get() as f32], [fw, fh], &self.queue);

        self.framebuffer = Framebuffer::new(
            (bw, bh),
            self.screen.filter,
            &self.device,
            &self.layouts.textured,
        );
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
        use {std::sync::mpsc, wgpu::*};

        const N_COLOR_CHANNELS: usize = 4;

        let image = ImageCopyTexture {
            texture: self.framebuffer.render_texture(),
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

            if Framebuffer::RENDER_FORMAT == TextureFormat::Bgra8UnormSrgb {
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

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
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

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) layers: Storage<Pipeline>,
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) views: Storage<Camera>,
    pub(crate) lights: Storage<Light>,
    pub(crate) spaces: Storage<LightSpace>,
}
