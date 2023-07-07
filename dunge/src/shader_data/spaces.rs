use {
    crate::{
        handles::SpacesHandle,
        layer::Layer,
        pipeline::Spaces as Bindings,
        render::Render,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::space::{Data, Format, Space, SpaceUniform},
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, Texture},
};

pub(crate) struct Spaces {
    group: u32,
    bind_group: BindGroup,
    #[allow(dead_code)]
    spaces: Buffer,
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
        }
    }

    pub fn update_data(
        &self,
        index: usize,
        data: Data,
        queue: &Queue,
    ) -> Result<(), UpdateDataError> {
        let texture = self.textures.get(index).ok_or(UpdateDataError::Index)?;
        texture.update_data(data, queue)
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

struct SpaceTexture {
    texture: Texture,
    size: (u8, u8, u8),
}

impl SpaceTexture {
    pub fn update_data(&self, data: Data, queue: &Queue) -> Result<(), UpdateDataError> {
        use wgpu::*;

        if data.size != self.size {
            return Err(UpdateDataError::Size);
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

#[derive(Debug)]
pub enum UpdateDataError {
    Index,
    Size,
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

    pub fn with_space(mut self, space: Space<'a>) -> Self {
        self.variables.textures_data.push(space.data);
        self.variables.light_spaces.push(space.into_uniform());
        self
    }

    pub fn build<S, T>(self, layer: &Layer<S, T>) -> SpacesHandle<S>
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
            .create_spaces(self.render, self.variables, layer)
    }
}
