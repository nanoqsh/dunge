use {
    crate::{
        layer::Layer,
        pipeline::Textures as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::texture::{Data as TextureData, InvalidSize, Texture},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::{BindGroup, BindGroupLayout, Queue},
};

pub struct Textures<S> {
    group: u32,
    bind_group: BindGroup,
    textures: Vec<Texture>,
    queue: Arc<Queue>,
    ty: PhantomData<S>,
}

impl<S> Textures<S> {
    fn new(params: Parameters, state: &State) -> Self {
        use {
            std::iter,
            wgpu::{
                AddressMode, BindGroupDescriptor, BindGroupEntry, BindingResource, FilterMode,
                SamplerDescriptor,
            },
        };

        let Parameters {
            bindings,
            variables,
            layout,
        } = params;

        let textures: Vec<_> = variables
            .maps
            .iter()
            .map(|&data| Texture::new(data, state))
            .collect();

        let mut entries = Vec::with_capacity(textures.len() + 1);
        for (texture, &binding) in iter::zip(&textures, &bindings.map.tdiffs) {
            entries.push(BindGroupEntry {
                binding,
                resource: BindingResource::TextureView(texture.view()),
            });
        }

        let device = state.device();
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        entries.push(BindGroupEntry {
            binding: bindings.map.sdiff,
            resource: BindingResource::Sampler(&sampler),
        });

        Self {
            group: bindings.group,
            bind_group: state.device().create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("texture bind group"),
            }),
            textures,
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    /// Updates the texture map with a new [data](`TextureData`).
    ///
    /// # Errors
    /// Will return [`InvalidSize`] if the size of the [data](`TextureData`)
    /// doesn't match the current texture size.
    ///
    /// # Panics
    /// Panics if the shader has no texture map with given index.
    pub fn update_map(&self, index: usize, data: TextureData) -> Result<(), InvalidSize>
    where
        S: Shader,
    {
        assert!(
            index < self.textures.len(),
            "the shader has no a texture map with index {index}",
        );

        self.textures[index].update(data, &self.queue)
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
    maps: Vec<TextureData<'a>>,
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
        self.variables.maps.push(data);
        self
    }

    /// Builds the textures.
    ///
    /// # Panics
    /// Panics if the shader requires texture `map`, but it's not set.
    #[must_use]
    pub fn build<S, T>(self, layer: &Layer<S, T>) -> Textures<S>
    where
        S: Shader,
    {
        let actual = self.variables.maps.len();
        let info = ShaderInfo::new::<S>();
        let expected = info.texture_maps();

        assert_eq!(
            actual, expected,
            "the shader requires {expected} texture maps, but {actual} is set",
        );

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
