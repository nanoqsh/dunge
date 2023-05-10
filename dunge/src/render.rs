use {
    crate::{
        bind_groups::Groups,
        canvas::{Backend as CanvasBackend, CanvasConfig, Error as CanvasError, Selector},
        context::Screenshot,
        frame::Frame,
        framebuffer::Framebuffer,
        pipeline::{Pipeline, PipelineParameters},
        r#loop::Loop,
        resources::Resources,
        screen::Screen,
        shader::Shader,
        shader_data::{Light, LightSpace, PostShaderData},
    },
    once_cell::unsync::OnceCell,
    wgpu::{
        Adapter, Device, Instance, Queue, ShaderModule, Surface, SurfaceConfiguration, SurfaceError,
    },
    winit::window::Window,
};

pub(crate) struct Render {
    context: RenderContext,
    surface_conf: SurfaceConfiguration,
    limits: RenderLimits,
    screen: Screen,
    shaders: Shaders,
    groups: Groups,
    post_pipeline: Pipeline,
    post_shader_data: PostShaderData,
    framebuffer: Framebuffer,
}

impl Render {
    pub fn new(context: RenderContext, resources: &mut Resources) -> Self {
        use wgpu::*;

        let surface_conf = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: Framebuffer::RENDER_FORMAT,
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        let limits = RenderLimits {
            max_texture_size: context.device.limits().max_texture_dimension_2d,
        };

        let groups = Groups::new(&context.device);
        let post_shader_data = PostShaderData::new(&context.device, &groups.post_shader_data);
        let shaders = Shaders::default();

        resources.lights.insert({
            const DEFAULT_AMBIENT: [f32; 3] = [1.; 3];

            Light::new(DEFAULT_AMBIENT, &[], &context.device, &groups.lights)
                .expect("default light")
        });

        resources.spaces.insert(
            LightSpace::new(&[], &[], &context.device, &context.queue, &groups.space)
                .expect("default light space"),
        );

        let post_pipeline = Pipeline::new(
            &context.device,
            &shaders,
            &groups,
            surface_conf.format,
            Shader::Post,
            PipelineParameters {
                blend: BlendState::ALPHA_BLENDING,
                topology: PrimitiveTopology::TriangleStrip,
                cull_faces: false,
                depth_stencil: None,
                ..Default::default()
            },
        );

        let framebuffer = Framebuffer::new_default(&context.device, &groups.textured);

        Self {
            context,
            surface_conf,
            limits,
            screen: Screen::default(),
            shaders,
            groups,
            post_pipeline,
            post_shader_data,
            framebuffer,
        }
    }

    pub fn drop_surface(&mut self) {
        self.context.surface.take();
    }

    pub fn recreate_surface(&mut self, window: &Window) {
        self.context.surface.get_or_insert_with(|| unsafe {
            let surface = self
                .context
                .instance
                .create_surface(&window)
                .expect("create surface");

            surface.configure(&self.context.device, &self.surface_conf);
            surface
        });
    }

    pub fn create_pipeline(&self, shader: Shader, params: PipelineParameters) -> Pipeline {
        Pipeline::new(
            &self.context.device,
            &self.shaders,
            &self.groups,
            self.surface_conf.format,
            shader,
            params,
        )
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        use std::num::NonZeroU32;

        self.set_screen({
            let (width, height) = size;
            let screen = self.screen();
            Some(Screen {
                width: NonZeroU32::new(width.max(1)).expect("non zero"),
                height: NonZeroU32::new(height.max(1)).expect("non zero"),
                ..screen
            })
        });
    }

    pub fn set_screen(&mut self, screen: Option<Screen>) {
        if let Some(screen) = screen {
            let (width, height) = screen.physical_size();
            if self.surface_conf.width == width && self.surface_conf.height == height {
                return;
            }

            self.screen = screen;
        }

        let (width, height) = self.screen.physical_size();
        self.surface_conf.width = width;
        self.surface_conf.height = height;
        self.context
            .surface
            .as_mut()
            .expect("surface")
            .configure(&self.context.device, &self.surface_conf);

        let (bw, bh) = self.screen.buffer_size(self.limits.max_texture_size);
        let (fw, fh) = self.screen.size_factor();
        self.post_shader_data.resize(
            [bw.get() as f32, bh.get() as f32],
            [fw, fh],
            &self.context.queue,
        );

        self.framebuffer = Framebuffer::new(
            (bw, bh),
            self.screen.filter,
            &self.context.device,
            &self.groups.textured,
        );
    }

    pub fn draw_frame<L>(&mut self, lp: &L, resources: &Resources) -> RenderResult<L::Error>
    where
        L: Loop,
    {
        use wgpu::*;

        let output = match self
            .context
            .surface
            .as_ref()
            .expect("surface")
            .get_current_texture()
        {
            Ok(output) => output,
            Err(err) => return RenderResult::SurfaceError(err),
        };

        let frame_view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut frame = Frame::new(self, resources, frame_view);
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
        let buffer = self.context.device.create_buffer(&BufferDescriptor {
            label: Some("copy buffer"),
            size: width as u64 * height as u64 * N_COLOR_CHANNELS as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let buffer = ImageCopyBuffer {
            buffer: &buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * N_COLOR_CHANNELS as u32),
                rows_per_image: Some(height),
            },
        };

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(image, buffer, size);
        self.context.queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel();
        let buffer_slice = buffer.buffer.slice(..);
        buffer_slice.map_async(MapMode::Read, move |res| _ = sender.send(res));

        self.context.device.poll(Maintain::Wait);
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

    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    pub fn groups(&self) -> &Groups {
        &self.groups
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
}

pub(crate) struct RenderContext {
    instance: Instance,
    device: Device,
    queue: Queue,
    surface: Option<Surface>,
}

impl RenderContext {
    pub async fn new(conf: CanvasConfig, window: &Window) -> Result<Self, CanvasError> {
        use wgpu::*;

        let instance = Instance::new(InstanceDescriptor {
            backends: match conf.backend {
                CanvasBackend::Vulkan => Backends::VULKAN,
                CanvasBackend::Gl => Backends::GL,
                CanvasBackend::Dx12 => Backends::DX12,
                CanvasBackend::Dx11 => Backends::DX11,
                CanvasBackend::Metal => Backends::METAL,
                CanvasBackend::WebGpu => Backends::BROWSER_WEBGPU,
            },
            dx12_shader_compiler: Dx12Compiler::default(),
        });

        // In Android a surface will be created later
        let surface = if cfg!(target_os = "android") {
            None
        } else {
            Some(unsafe { instance.create_surface(window).expect("create surface") })
        };

        let (device, queue) = {
            let adapter = Self::select_adapter(conf.selector, &instance, surface.as_ref())
                .await
                .ok_or(CanvasError::BackendSelection)?;

            let backend = adapter.get_info().backend;
            log::info!("selected backend: {backend:?}");

            let desc = DeviceDescriptor {
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
            };

            let Ok(dev) = adapter.request_device(&desc, None).await else {
                return Err(CanvasError::RequestDevice);
            };

            dev
        };

        Ok(Self {
            instance,
            device,
            queue,
            surface,
        })
    }

    async fn select_adapter(
        selector: Selector,
        instance: &Instance,
        surface: Option<&Surface>,
    ) -> Option<Adapter> {
        match selector {
            Selector::Auto => {
                use wgpu::{PowerPreference, RequestAdapterOptions};

                instance
                    .request_adapter(&RequestAdapterOptions {
                        power_preference: PowerPreference::HighPerformance,
                        force_fallback_adapter: false,
                        compatible_surface: surface,
                    })
                    .await
            }
            #[cfg(not(target_arch = "wasm32"))]
            Selector::Callback(mut callback) => {
                use {
                    crate::canvas::{Device, SelectorEntry},
                    wgpu::{Backends, DeviceType},
                };

                let mut adapters = vec![];
                let mut entries = vec![];
                for adapter in instance.enumerate_adapters(Backends::all()) {
                    let info = adapter.get_info();
                    let entry = SelectorEntry {
                        name: info.name,
                        device: match info.device_type {
                            DeviceType::IntegratedGpu => Device::IntegratedGpu,
                            DeviceType::DiscreteGpu => Device::DiscreteGpu,
                            DeviceType::VirtualGpu => Device::VirtualGpu,
                            DeviceType::Cpu => Device::Cpu,
                            DeviceType::Other => panic!("undefined device type"),
                        },
                    };

                    adapters.push(adapter);
                    entries.push(entry);
                }

                let selected = callback(entries)?;
                (selected < adapters.len()).then(|| adapters.swap_remove(selected))
            }
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

struct RenderLimits {
    max_texture_size: u32,
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
