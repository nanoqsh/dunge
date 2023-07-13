use {
    crate::{
        pipeline::{Parameters, Pipeline},
        shader_data::PostShaderData,
    },
    dunge_shader::TextureBindings,
    wgpu::{BindGroup, BindGroupLayout, Device, Queue, RenderPipeline},
};

pub(crate) struct PostProcessor {
    data: PostShaderData,
    pipeline: Pipeline,
    antialiasing: bool,
}

impl PostProcessor {
    pub const DATA_GROUP: u32 = 0;
    pub const DATA_BINDING: u32 = 0;
    pub const TEXTURE_GROUP: u32 = 1;
    pub const TEXTURE_TDIFF_BINDING: u32 = 0;
    pub const TEXTURE_SDIFF_BINDING: u32 = 1;

    pub fn new(device: &Device) -> Self {
        const DEFAULT_ANTIALIASING: bool = false;

        let pipeline = Self::pipeline(device, DEFAULT_ANTIALIASING);
        let globals = &pipeline.globals().expect("globals").layout;
        let data = PostShaderData::new(device, globals);
        Self {
            data,
            pipeline,
            antialiasing: DEFAULT_ANTIALIASING,
        }
    }

    pub fn set_antialiasing(&mut self, device: &Device, antialiasing: bool) {
        if self.antialiasing == antialiasing {
            return;
        }

        self.pipeline = Self::pipeline(device, antialiasing);
    }

    pub fn resize(&self, size: (f32, f32), factor: (f32, f32), queue: &Queue) {
        self.data.resize(size, factor, queue);
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.pipeline.textures().expect("textures layout").layout
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref()
    }

    pub fn data_bind_group(&self) -> &BindGroup {
        self.data.bind_group()
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
                    tdiffs: vec![Self::TEXTURE_TDIFF_BINDING],
                    sdiff: Self::TEXTURE_SDIFF_BINDING,
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
