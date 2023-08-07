use {
    crate::{
        canvas::{Backend as CanvasBackend, CanvasConfig, Error as CanvasError, Info, Selector},
        context::Screenshot,
        frame::{Frame, Snapshot},
        framebuffer::{BufferSize, Framebuffer},
        postproc::PostProcessor,
        r#loop::Loop,
        screen::{RenderScreen, Screen},
        shader_data::{Instance as ShaderInstance, ModelTransform},
    },
    std::{ops, sync::Arc},
    wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, SurfaceError},
    winit::window::Window,
};

pub(crate) struct Render {
    state: State,
    conf: SurfaceConfiguration,
    screen: RenderScreen,
    postproc: PostProcessor,
    framebuffer: Framebuffer,
    instance: ShaderInstance,
}

impl Render {
    pub fn new(state: State) -> Self {
        use wgpu::*;

        let conf = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: Framebuffer::RENDER_FORMAT,
            width: 1,
            height: 1,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        let screen = RenderScreen::new(&state);
        let postproc = PostProcessor::new(&state);
        let framebuffer = Framebuffer::new(&state);
        let instance = ShaderInstance::new(&[ModelTransform::default()], &state);

        Self {
            state,
            conf,
            screen,
            postproc,
            framebuffer,
            instance,
        }
    }

    pub fn info(&self) -> &Info {
        &self.state.info
    }

    pub fn drop_surface(&mut self) {
        self.state.surface.take();
    }

    pub fn recreate_surface(&mut self, window: &Window) {
        self.state.surface.get_or_insert_with(|| unsafe {
            let surface = self
                .state
                .instance
                .create_surface(&window)
                .expect("create surface");

            surface.configure(&self.state.device, &self.conf);
            surface
        });
    }

    pub fn screen(&self) -> Screen {
        self.screen.screen()
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        use std::num::NonZeroU32;

        self.set_screen({
            let (width, height) = size;
            Screen {
                width: NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
                height: NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN),
                ..self.screen()
            }
        });
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.screen.set_screen(screen);

        let (width, height) = screen.physical_size().into();
        self.conf.width = width;
        self.conf.height = height;
        self.state
            .surface
            .as_mut()
            .expect("surface")
            .configure(&self.state.device, &self.conf);

        self.framebuffer
            .set_size(self.screen.buffer_size(), &self.state);
    }

    pub fn draw_frame<L>(&mut self, lp: &L) -> Result<(), SurfaceError>
    where
        L: Loop,
    {
        use wgpu::*;

        let output = self
            .state
            .surface
            .as_ref()
            .expect("surface")
            .get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut frame = Frame::new(Snapshot::new(self), view);
        lp.render(&mut frame);
        frame.draw_on_screen();
        output.present();
        Ok(())
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

        let BufferSize(width, height) = self.screen.buffer_size();
        let buffer = self.state.device.create_buffer(&BufferDescriptor {
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
            .state
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(image, buffer, size);
        self.state.queue.submit([encoder.finish()]);

        let (sender, receiver) = mpsc::channel();
        let buffer_slice = buffer.buffer.slice(..);
        buffer_slice.map_async(MapMode::Read, move |res| _ = sender.send(res));

        self.state.device.poll(Maintain::Wait);
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

        let virtual_size = self.screen().virtual_size_with_antialiasing();
        let data = {
            let view = buffer_slice.get_mapped_range();
            let mut data = Vec::with_capacity(
                virtual_size.x as usize * virtual_size.y as usize * N_COLOR_CHANNELS,
            );

            let row_size = width as usize * N_COLOR_CHANNELS;
            let virt_row_size = virtual_size.x as usize * N_COLOR_CHANNELS;
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
            width: virtual_size.x,
            height: virtual_size.y,
            data,
        }
    }
}

impl ops::Deref for Render {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'d> Snapshot<'d> {
    fn new(render: &'d mut Render) -> Self {
        Self {
            state: &render.state,
            framebuffer: &render.framebuffer,
            postproc: &mut render.postproc,
            screen: render.screen,
            instance: &render.instance,
        }
    }
}

pub(crate) struct State {
    info: Info,
    instance: Instance,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Option<Surface>,
}

impl State {
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

        let (device, queue, info) = {
            let adapter = Self::select_adapter(conf.selector, &instance, surface.as_ref())
                .await
                .ok_or(CanvasError::BackendSelection(conf.backend))?;

            let info = Info::from_adapter_info(adapter.get_info());
            let backend = info.backend;
            log::info!("selected backend: {backend:?}");

            let desc = DeviceDescriptor {
                features: Features::empty(),
                limits: Limits {
                    max_texture_dimension_2d: 4096,
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

            let (device, queue) = adapter
                .request_device(&desc, None)
                .await
                .map_err(|_| CanvasError::RequestDevice)?;

            (device, queue, info)
        };

        Ok(Self {
            info,
            instance,
            device: Arc::new(device),
            queue: Arc::new(queue),
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
                use wgpu::Backends;

                let mut adapters = vec![];
                let mut entries = vec![];
                for adapter in instance.enumerate_adapters(Backends::all()) {
                    let info = adapter.get_info();
                    adapters.push(adapter);
                    entries.push(Info::from_adapter_info(info));
                }

                let selected = callback(entries)?;
                (selected < adapters.len()).then(|| adapters.swap_remove(selected))
            }
        }
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    pub fn queue(&self) -> &Arc<Queue> {
        &self.queue
    }
}
