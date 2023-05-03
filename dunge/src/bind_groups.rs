use {
    crate::shader::{self, Shader},
    wgpu::{BindGroupLayout, Device},
};

pub(crate) struct Layouts {
    pub textured: BindGroupLayout,
    pub camera: BindGroupLayout,
    pub post_shader_data: BindGroupLayout,
    pub lights: BindGroupLayout,
    pub space: BindGroupLayout,
}

impl Layouts {
    pub fn new(device: &Device) -> Self {
        use wgpu::*;

        Self {
            textured: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: shader::TDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture bind group layout"),
            }),
            camera: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: shader::CAMERA_BINDING,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera bind group layout"),
            }),
            post_shader_data: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: shader::DATA_BINDING,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("post shader data bind group layout"),
            }),
            lights: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: shader::AMBIENT_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SOURCES_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("lights bind group layout"),
            }),
            space: device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: shader::SPACES_BINDING,
                        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SPACE0_TDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D3,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SPACE1_TDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D3,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SPACE2_TDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D3,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SPACE3_TDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D3,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: shader::SPACE_SDIFF_BINDING,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("space bind group layout"),
            }),
        }
    }

    pub fn bind_group_layouts(&self, shader: Shader) -> BindGroupLayouts {
        match shader {
            Shader::Color => BindGroupLayouts::N2([&self.camera, &self.lights]),
            Shader::Flat => BindGroupLayouts::N1([&self.textured]),
            Shader::Post => BindGroupLayouts::N2([&self.post_shader_data, &self.textured]),
            Shader::Textured => {
                BindGroupLayouts::N4([&self.camera, &self.textured, &self.lights, &self.space])
            }
        }
    }
}

pub(crate) enum BindGroupLayouts<'a> {
    N1([&'a BindGroupLayout; 1]),
    N2([&'a BindGroupLayout; 2]),
    N4([&'a BindGroupLayout; 4]),
}

impl<'a> BindGroupLayouts<'a> {
    pub fn as_slice(&self) -> &[&'a BindGroupLayout] {
        match self {
            Self::N1(b) => b,
            Self::N2(b) => b,
            Self::N4(b) => b,
        }
    }
}
