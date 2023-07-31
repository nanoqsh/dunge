use {
    crate::{
        framebuffer::BufferSize,
        pipeline::{Parameters, Pipeline},
        render::State,
        shader_data::PostShaderData,
    },
    dunge_shader::{PostScheme, TextureBindings, Vignette},
    glam::Vec2,
    std::sync::OnceLock,
    wgpu::{BindGroup, FilterMode, RenderPipeline, Sampler, TextureView},
};

/// Describes a frame render filter mode.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum FrameFilter {
    #[default]
    Nearest,
    Linear,
}

impl FrameFilter {
    fn mode(self) -> FilterMode {
        match self {
            Self::Nearest => FilterMode::Nearest,
            Self::Linear => FilterMode::Linear,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct FrameParameters {
    pub buffer_size: BufferSize,
    pub factor: Vec2,
    pub filter: FrameFilter,
    pub antialiasing: bool,
    pub vignette: Vignette,
}

pub(crate) struct PostProcessor {
    data: PostShaderData,
    pipeline: Pipeline,
    bind_group: OnceLock<BindGroup>,
    sampler: Sampler,
    params: FrameParameters,
}

impl PostProcessor {
    pub const DATA_GROUP: u32 = 0;
    pub const DATA_BINDING: u32 = 0;
    pub const TEXTURE_GROUP: u32 = 1;
    const TEXTURE_TDIFF_BINDING: u32 = 0;
    const TEXTURE_SDIFF_BINDING: u32 = 1;

    pub fn new(state: &State) -> Self {
        let params = FrameParameters::default();
        let FrameParameters {
            buffer_size,
            factor,
            filter,
            antialiasing,
            vignette,
        } = params;

        let pipeline = Self::pipeline(state, antialiasing, vignette);
        let globals = &pipeline.globals().expect("globals").layout;
        let data = PostShaderData::new(state, globals);
        data.resize(buffer_size.into(), factor.into());

        Self {
            data,
            pipeline,
            bind_group: OnceLock::new(),
            sampler: Self::sampler(state, filter),
            params,
        }
    }

    pub fn set_parameters(&mut self, state: &State, params: FrameParameters) {
        let FrameParameters {
            buffer_size,
            factor,
            filter,
            antialiasing,
            vignette,
        } = params;

        if self.params.antialiasing != antialiasing || self.params.vignette != vignette {
            self.pipeline = Self::pipeline(state, antialiasing, vignette);
        }

        if self.params.filter != filter {
            self.sampler = Self::sampler(state, filter);
        }

        if self.params.factor != factor {
            self.data.resize(buffer_size.into(), factor.into());
        }

        if self.params.buffer_size != buffer_size {
            self.bind_group.take();
        }

        self.params = params;
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref()
    }

    pub fn data_bind_group(&self) -> &BindGroup {
        self.data.bind_group()
    }

    pub fn render_bind_group(&self, state: &State, view: &TextureView) -> &BindGroup {
        use wgpu::*;

        self.bind_group.get_or_init(|| {
            state.device().create_bind_group(&BindGroupDescriptor {
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
            })
        })
    }

    fn sampler(state: &State, filter: FrameFilter) -> Sampler {
        use wgpu::{AddressMode, SamplerDescriptor};

        let mode = filter.mode();
        state.device().create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: mode,
            min_filter: mode,
            ..Default::default()
        })
    }

    fn pipeline(state: &State, antialiasing: bool, vignette: Vignette) -> Pipeline {
        use {
            dunge_shader::Shader,
            wgpu::{BlendState, PrimitiveTopology},
        };

        let scheme = PostScheme {
            post_data: Self::DATA_BINDING,
            map: TextureBindings {
                tmaps: vec![Self::TEXTURE_TDIFF_BINDING],
                smap: Self::TEXTURE_SDIFF_BINDING,
            },
            antialiasing,
            vignette,
        };

        let shader = Shader::postproc(scheme);
        log::debug!("generated post shader:\n{src}", src = shader.source);

        Pipeline::new(
            state,
            &shader,
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
