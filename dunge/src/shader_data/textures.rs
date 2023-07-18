use {
    crate::{
        layer::Layer,
        pipeline::Textures as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::{data::TextureData, texture::Texture},
    },
    std::marker::PhantomData,
    wgpu::{BindGroup, BindGroupLayout},
};

/// Shader textures.
///
/// Can be created from the [context](crate::Context) by calling
/// the [`textures_builder`](crate::Context::textures_builder) function.
pub struct Textures<S> {
    group: u32,
    bind_group: BindGroup,
    textures: Vec<Texture>,
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
            .into_iter()
            .map(|map| match map {
                Map::Data(data) => Texture::new(data, state),
                Map::Texture(texture) => texture,
            })
            .collect();

        let mut entries = Vec::with_capacity(textures.len() + 1);
        for (texture, &binding) in iter::zip(&textures, &bindings.map.tmaps) {
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
            binding: bindings.map.smap,
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
            ty: PhantomData,
        }
    }

    /// Returns the [texture](Texture) map by index.
    ///
    /// # Panics
    /// Panics if the shader has no texture map with given index.
    pub fn get_map(&self, index: usize) -> &Texture
    where
        S: Shader,
    {
        assert!(
            index < self.textures.len(),
            "the shader has no a texture map with index {index}",
        );

        &self.textures[index]
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
    maps: Vec<Map<'a>>,
}

/// The [textures](Textures) builder.
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

    /// Sets a texture map for the textures object.
    pub fn with_map<M>(mut self, map: M) -> Self
    where
        M: Into<Map<'a>>,
    {
        self.variables.maps.push(map.into());
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

/// The texture map parameter.
pub enum Map<'a> {
    Data(TextureData<'a>),
    Texture(Texture),
}

impl<'a> From<TextureData<'a>> for Map<'a> {
    fn from(v: TextureData<'a>) -> Self {
        Self::Data(v)
    }
}

impl From<Texture> for Map<'_> {
    fn from(v: Texture) -> Self {
        Self::Texture(v)
    }
}
