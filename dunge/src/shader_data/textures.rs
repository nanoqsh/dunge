use {
    crate::{
        error::InvalidSize,
        layer::Layer,
        pipeline::Textures as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::texture::{Data as TextureData, Texture},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::{BindGroup, BindGroupLayout, Queue},
};

pub struct Textures<S> {
    group: u32,
    bind_group: BindGroup,
    map: Option<Texture>,
    queue: Arc<Queue>,
    ty: PhantomData<S>,
}

impl<S> Textures<S> {
    fn new(params: Parameters, state: &State) -> Self {
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
            ty: PhantomData,
        }
    }

    pub fn update_map(&self, data: TextureData) -> Result<(), InvalidSize>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        assert!(info.has_map, "the shader has no texture map");

        self.map
            .as_ref()
            .expect("texture map")
            .update(data, &self.queue)
    }

    pub(crate) fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

struct Parameters<'a> {
    variables: Variables<'a>,
    bindings: &'a Bindings,
    layout: &'a BindGroupLayout,
}

#[derive(Default)]
struct Variables<'a> {
    map: Option<TextureData<'a>>,
}

#[must_use]
pub struct Builder<'a> {
    state: &'a State,
    variables: Variables<'a>,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self {
            state,
            variables: Variables::default(),
        }
    }

    pub fn with_map(mut self, data: TextureData<'a>) -> Self {
        self.variables.map = Some(data);
        self
    }

    #[must_use]
    pub fn build<S, T>(self, layer: &Layer<S, T>) -> Textures<S>
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

        let textures = layer
            .pipeline()
            .textures()
            .expect("the shader has no textures");

        let params = Parameters {
            variables: self.variables,
            bindings: &textures.bindings,
            layout: &textures.layout,
        };

        Textures::new(params, self.state)
    }
}
