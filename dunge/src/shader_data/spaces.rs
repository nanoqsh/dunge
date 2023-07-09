use {
    crate::{
        layer::Layer,
        pipeline::Spaces as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::space::{Data, Format, Space, SpaceUniform},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Queue, Texture},
};

pub struct Spaces<S> {
    group: u32,
    bind_group: BindGroup,
    #[allow(dead_code)]
    spaces: Buffer,
    textures: Box<[SpaceTexture]>,
    queue: Arc<Queue>,
    ty: PhantomData<S>,
}

impl<S> Spaces<S> {
    fn new(params: Parameters, state: &State) -> Self {
        use {
            std::iter,
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                *,
            },
        };

        let Parameters {
            variables,
            bindings,
            layout,
        } = params;

        let device = state.device();
        let spaces = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("spaces buffer"),
            contents: bytemuck::cast_slice(&variables.light_spaces),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut views = vec![];
        let textures: Box<[_]> = variables
            .textures_data
            .into_iter()
            .map(|Data { data, size, format }| {
                let (width, height, depth) = size;
                let size = Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: depth as u32,
                };

                let texture = device.create_texture(&TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D3,
                    format: match format {
                        Format::Srgba => TextureFormat::Rgba8UnormSrgb,
                        Format::Rgba => TextureFormat::Rgba8Unorm,
                        Format::Gray => TextureFormat::R8Unorm,
                    },
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                state.queue().write_texture(
                    ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: Origin3d::ZERO,
                        aspect: TextureAspect::All,
                    },
                    data,
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(format.n_channels() as u32 * width as u32),
                        rows_per_image: Some(height as u32),
                    },
                    size,
                );

                views.push(texture.create_view(&TextureViewDescriptor::default()));
                SpaceTexture {
                    texture,
                    size: (width, height, depth),
                }
            })
            .collect();

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let mut entries = vec![BindGroupEntry {
            binding: bindings.bindings.spaces,
            resource: spaces.as_entire_binding(),
        }];

        entries.extend(
            iter::zip(&views, &bindings.bindings.tdiffs).map(|(view, &tdiff)| BindGroupEntry {
                binding: tdiff,
                resource: BindingResource::TextureView(view),
            }),
        );

        entries.push(BindGroupEntry {
            binding: bindings.bindings.sdiff,
            resource: BindingResource::Sampler(&sampler),
        });

        Self {
            group: bindings.group,
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("spaces bind group"),
            }),
            spaces,
            textures,
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    /// Updates the spaces with a new [data](`Data`).
    ///
    /// # Errors
    /// Will return [`UpdateError::Index`] if the index is invalid or
    /// [`UpdateError::Size`] if the [data](`Data`) doesn't match the current size.
    ///
    /// # Panics
    /// Panics if the shader has no light spaces.
    pub fn update(&self, index: usize, data: Data) -> Result<(), UpdateError>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        assert!(
            !info.light_spaces.is_empty(),
            "the shader has no light spaces",
        );

        let texture = self.textures.get(index).ok_or(UpdateError::Index)?;
        texture.update(data, &self.queue)
    }

    pub(crate) fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

struct SpaceTexture {
    texture: Texture,
    size: (u8, u8, u8),
}

impl SpaceTexture {
    fn update(&self, data: Data, queue: &Queue) -> Result<(), UpdateError> {
        use wgpu::*;

        if data.size != self.size {
            return Err(UpdateError::Size);
        }

        let (width, height, depth) = self.size;
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(data.format.n_channels() as u32 * width as u32),
                rows_per_image: Some(height as u32),
            },
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: depth as u32,
            },
        );

        Ok(())
    }
}

/// An error returned from the [`update`](Spaces::update) function.
#[derive(Debug)]
pub enum UpdateError {
    /// The index is invalid.
    Index,

    /// [`Data`] size doesn't match the current size.
    Size,
}

struct Parameters<'a> {
    variables: Variables<'a>,
    bindings: &'a Bindings,
    layout: &'a BindGroupLayout,
}

#[derive(Default)]
struct Variables<'a> {
    light_spaces: Vec<SpaceUniform>,
    textures_data: Vec<Data<'a>>,
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

    /// Set a [space](Space) to shader.
    pub fn with_space(mut self, space: Space<'a>) -> Self {
        self.variables.textures_data.push(space.data);
        self.variables.light_spaces.push(space.into_uniform());
        self
    }

    /// Builds the spaces.
    ///
    /// # Panics
    /// Panics if the shader requires light spaces, but they aren't set
    /// or some light space format doesn't match.
    #[must_use]
    pub fn build<S, T>(self, layer: &Layer<S, T>) -> Spaces<S>
    where
        S: Shader,
    {
        let light_spaces = ShaderInfo::new::<S>().light_spaces;
        let actual = self.variables.light_spaces.len();
        let expected = light_spaces.len();

        assert_eq!(
            actual, expected,
            "the shader requires {expected} light spaces, but {actual} is set",
        );

        for ((n, kind), Data { format, .. }) in
            light_spaces.enumerate().zip(&self.variables.textures_data)
        {
            assert!(
                format.matches(kind),
                "light space format {format} ({n}) doesn't match",
            );
        }

        let spaces = layer.pipeline().spaces().expect("the shader has no spaces");
        let params = Parameters {
            variables: self.variables,
            bindings: &spaces.bindings,
            layout: &spaces.layout,
        };

        Spaces::new(params, self.state)
    }
}
