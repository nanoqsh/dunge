use {
    crate::{
        pipeline::{Parameters, Pipeline},
        shader_data::PostShaderData,
    },
    dunge_shader::{Globals, TextureBindings, Textures},
    wgpu::{BindGroup, Device, Queue, RenderPipeline},
};

pub(crate) struct PostProcessor {
    pipeline: Pipeline,
    data: PostShaderData,
}

impl PostProcessor {
    pub const DATA_GROUP: u32 = 0;
    pub const DATA_BINDING: u32 = 0;
    pub const TEXTURE_GROUP: u32 = 1;
    pub const TEXTURE_BINDING: TextureBindings = TextureBindings { tdiff: 0, sdiff: 1 };

    pub fn new(device: &Device) -> Self {
        use {
            dunge_shader::{Layout, ShaderInfo},
            wgpu::{BlendState, PrimitiveTopology},
        };

        let pipeline = Pipeline::new(
            device,
            &ShaderInfo {
                layout: Layout {
                    globals: Globals {
                        post_data: Some(Self::DATA_BINDING),
                        ..Default::default()
                    },
                    textures: Textures {
                        map: Some(Self::TEXTURE_BINDING),
                        ..Default::default()
                    },
                },
                source: String::from(include_str!("shaders/post.wgsl")),
            },
            None,
            Parameters {
                blend: BlendState::ALPHA_BLENDING,
                topology: PrimitiveTopology::TriangleStrip,
                cull_faces: false,
                depth_stencil: None,
                ..Default::default()
            },
        );

        let layout = &pipeline.globals().expect("globals").layout;
        let data = PostShaderData::new(device, layout);
        Self { pipeline, data }
    }

    pub fn resize(&self, size: [f32; 2], factor: [f32; 2], queue: &Queue) {
        self.data.resize(size, factor, queue);
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        self.pipeline.as_ref()
    }

    pub fn data_bind_group(&self) -> &BindGroup {
        self.data.bind_group()
    }
}
