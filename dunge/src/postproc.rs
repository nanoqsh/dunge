use {
    crate::{
        framebuffer::BufferSize,
        pipeline::{Parameters, Pipeline},
        render::State,
        screen::RenderScreen,
        shader_data::PostShaderData,
    },
    dunge_shader::TextureBindings,
    wgpu::{BindGroup, Device, Queue, RenderPipeline, Sampler, TextureView},
};

/// Describes a frame render filter mode.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum FrameFilter {
    #[default]
    Nearest,
    Linear,
}

pub(crate) struct PostProcessor {
    data: PostShaderData,
    pipeline: Pipeline,
    bind_group: BindGroup,
    sampler: Sampler,
    antialiasing: bool,
    filter: FrameFilter,
}

impl PostProcessor {
    pub const DATA_GROUP: u32 = 0;
    pub const DATA_BINDING: u32 = 0;
    pub const TEXTURE_GROUP: u32 = 1;
    const TEXTURE_TDIFF_BINDING: u32 = 0;
    const TEXTURE_SDIFF_BINDING: u32 = 1;

    pub fn new(state: &State, view: &TextureView, screen: RenderScreen) -> Self {
        use wgpu::*;

        let buffer_size = screen.buffer_size();
        let screen = screen.screen();
        let antialiasing = screen.is_antialiasing_enabled();
        let filter = screen.filter;

        let device = state.device();
        let pipeline = Self::pipeline(device, antialiasing);
        let globals = &pipeline.globals().expect("globals").layout;
        let data = PostShaderData::new(device, globals);
        data.resize(buffer_size, screen.size_factor().into(), state.queue());

        let sampler = Self::sampler(device, filter);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.textures().expect("textures layout").layout,
            entries: &[
                BindGroupEntry {
                    binding: Self::TEXTURE_TDIFF_BINDING,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: Self::TEXTURE_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            data,
            pipeline,
            bind_group,
            sampler,
            antialiasing,
            filter,
        }
    }

    pub fn set_view(&mut self, device: &Device, view: &TextureView) {
        use wgpu::*;

        self.bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.pipeline.textures().expect("textures layout").layout,
            entries: &[
                BindGroupEntry {
                    binding: Self::TEXTURE_TDIFF_BINDING,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: Self::TEXTURE_SDIFF_BINDING,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        });
    }

    pub fn set_antialiasing(&mut self, device: &Device, antialiasing: bool) {
        if self.antialiasing == antialiasing {
            return;
        }

        self.pipeline = Self::pipeline(device, antialiasing);
    }

    pub fn set_filter(&mut self, device: &Device, filter: FrameFilter) {
        if self.filter == filter {
            return;
        }

        self.sampler = Self::sampler(device, filter);
    }

    pub fn resize(&self, size: BufferSize, factor: (f32, f32), queue: &Queue) {
        self.data.resize(size, factor, queue);
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref()
    }

    pub fn data_bind_group(&self) -> &BindGroup {
        self.data.bind_group()
    }

    pub fn render_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn sampler(device: &Device, filter: FrameFilter) -> Sampler {
        use wgpu::{AddressMode, FilterMode, SamplerDescriptor};

        let filter_mode = match filter {
            FrameFilter::Nearest => FilterMode::Nearest,
            FrameFilter::Linear => FilterMode::Linear,
        };

        device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            ..Default::default()
        })
    }

    fn pipeline(device: &Device, antialiasing: bool) -> Pipeline {
        use {
            dunge_shader::Shader,
            wgpu::{BlendState, PrimitiveTopology},
        };

        Pipeline::new(
            device,
            &Shader::postproc(
                Self::DATA_BINDING,
                TextureBindings {
                    tmaps: vec![Self::TEXTURE_TDIFF_BINDING],
                    smap: Self::TEXTURE_SDIFF_BINDING,
                },
                if antialiasing {
                    String::from(include_str!("shaders/post_ssaa.wgsl"))
                } else {
                    String::from(include_str!("shaders/post.wgsl"))
                },
            ),
            None,
            Parameters {
                blend: BlendState::ALPHA_BLENDING,
                topology: PrimitiveTopology::TriangleStrip,
                cull_faces: false,
                depth_stencil: None,
                ..Default::default()
            },
        )
    }
}
