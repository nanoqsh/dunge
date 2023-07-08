use {
    crate::{
        error::TooLargeSize,
        handles::TexturesHandle,
        layer::Layer,
        pipeline::Textures as Bindings,
        render::State,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::texture::{Data as TextureData, Texture},
    },
    std::sync::Arc,
    wgpu::{BindGroup, BindGroupLayout, Queue},
};

pub(crate) struct Textures {
    group: u32,
    bind_group: BindGroup,
    map: Option<Texture>,
    queue: Arc<Queue>,
}

impl Textures {
    pub fn new(params: Parameters, state: &State) -> Self {
        use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

        let Parameters {
            bindings,
            variables,
            layout,
        } = params;

        let map = variables.map.map(|data| Texture::new(data, state));
        let entries = map.as_ref().map(|texture| {
            [
                BindGroupEntry {
                    binding: bindings.map.tdiff,
                    resource: BindingResource::TextureView(texture.view()),
                },
                BindGroupEntry {
                    binding: bindings.map.sdiff,
                    resource: BindingResource::Sampler(texture.sampler()),
                },
            ]
        });

        Self {
            group: bindings.group,
            bind_group: state.device().create_bind_group(&BindGroupDescriptor {
                layout,
                entries: match &entries {
                    Some(bind) => bind,
                    None => &[],
                },
                label: Some("texture bind group"),
            }),
            map,
            queue: Arc::clone(state.queue()),
        }
    }

    pub fn update_data(&self, data: TextureData) -> Result<(), TooLargeSize> {
        self.map
            .as_ref()
            .expect("texture map")
            .update(data, &self.queue)
    }

    pub fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

pub(crate) struct Parameters<'a> {
    pub variables: Variables<'a>,
    pub bindings: &'a Bindings,
    pub layout: &'a BindGroupLayout,
}

#[derive(Default)]
pub(crate) struct Variables<'a> {
    pub map: Option<TextureData<'a>>,
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    state: &'a State,
    variables: Variables<'a>,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, state: &'a State) -> Self {
        Self {
            resources,
            state,
            variables: Variables::default(),
        }
    }

    pub fn with_map(mut self, data: TextureData<'a>) -> Self {
        self.variables.map = Some(data);
        self
    }

    pub fn build<S, T>(self, handle: &Layer<S, T>) -> TexturesHandle<S>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        if info.has_map {
            assert!(
                self.variables.map.is_some(),
                "the shader requires texture `map`, but it's not set",
            );
        }

        self.resources
            .create_textures(self.state, self.variables, handle)
    }
}
