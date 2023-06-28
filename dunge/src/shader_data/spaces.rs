use {
    crate::{
        color::IntoLinear,
        error::ResourceNotFound,
        handles::{LayerHandle, SpacesHandle},
        layout::Plain,
        pipeline::Spaces as Bindings,
        render::Render,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::space::{Data, Format, Space, SpaceUniform},
        transform::IntoMat,
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture},
};

pub(crate) struct Spaces {
    group: u32,
    bind_group: BindGroup,
    spaces: SpacesBuffer,
    textures: Box<[SpaceTexture]>,
}

impl Spaces {
    pub fn new(params: Parameters, device: &Device, queue: &Queue) -> Self {
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

        let spaces = SpacesBuffer {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("spaces buffer"),
                contents: variables.light_spaces.as_slice().as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
            size: variables.light_spaces.len() as u32,
        };

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

                queue.write_texture(
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
            resource: spaces.buffer.as_entire_binding(),
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
        }
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
    pub light_spaces: Vec<SpaceUniform>,
    pub textures_data: Vec<Data<'a>>,
}

struct SpacesBuffer {
    buffer: Buffer,
    size: u32,
}

struct SpaceTexture {
    texture: Texture,
    size: (u8, u8, u8),
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    render: &'a Render,
    variables: Variables<'a>,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, render: &'a Render) -> Self {
        Self {
            resources,
            render,
            variables: Variables::default(),
        }
    }

    pub fn with_space<M, C>(mut self, space: Space<'a, M, C>) -> Self
    where
        M: IntoMat,
        C: IntoLinear<3>,
    {
        self.variables.textures_data.push(space.data);
        self.variables.light_spaces.push(space.into_uniform());
        self
    }

    pub fn build<S>(self, handle: LayerHandle<S>) -> Result<SpacesHandle<S>, ResourceNotFound>
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

        self.resources
            .create_spaces(self.render, self.variables, handle)
    }
}
