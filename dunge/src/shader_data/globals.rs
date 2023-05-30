use {
    crate::{
        camera::{Camera, Projection, View},
        color::{IntoLinear, Linear},
        error::ResourceNotFound,
        handles::{GlobalsHandle, LayerHandle},
        layout::Plain,
        pipeline::GlobalsBindings as Bindings,
        render::Render,
        resources::Resources,
        shader::Shader,
        shader_data::{AmbientUniform, CameraUniform},
    },
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Globals {
    camera: Option<(Camera, Buffer)>,
    ambient: Option<Buffer>,
    bind_group: BindGroup,
}

impl Globals {
    pub fn new(params: Parameters, device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let Parameters {
            bindings,
            uniforms,
            layout,
            ..
        } = params;

        let camera = uniforms.camera.map(|uniform| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        });

        let ambient = uniforms.ambient.map(|uniform| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: uniform.as_bytes(),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        });

        let entries: Vec<_> = [
            camera.as_ref().map(|buf| BindGroupEntry {
                binding: bindings.camera,
                resource: buf.as_entire_binding(),
            }),
            ambient.as_ref().map(|buf| BindGroupEntry {
                binding: bindings.ambient,
                resource: buf.as_entire_binding(),
            }),
        ]
        .into_iter()
        .flatten()
        .collect();

        Self {
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("globals bind group"),
            }),
            camera: camera.map(|buf| (params.camera, buf)),
            ambient,
        }
    }

    pub fn set_view(&mut self, view: View<Projection>) {
        let (camera, _) = self.camera.as_mut().expect("camera");
        camera.set_view(view);
    }

    pub fn write_camera(&self, size: (u32, u32), queue: &Queue) {
        let Some((camera, buf)) = &self.camera else {
            return;
        };

        let uniform = camera.uniform(size);
        queue.write_buffer(buf, 0, uniform.as_bytes());
    }

    pub fn write_ambient(&self, color: [f32; 3], queue: &Queue) {
        let Some(buf) = &self.ambient else {
            return;
        };

        let uniform = AmbientUniform::new(color);
        queue.write_buffer(buf, 0, uniform.as_bytes());
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub(crate) struct Parameters<'a> {
    pub camera: Camera,
    pub bindings: Bindings,
    pub uniforms: Uniforms,
    pub layout: &'a BindGroupLayout,
}

#[derive(Default)]
pub(crate) struct Uniforms {
    pub camera: Option<CameraUniform>,
    pub ambient: Option<AmbientUniform>,
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    render: &'a Render,
    uniforms: Uniforms,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, render: &'a Render) -> Self {
        Self {
            resources,
            render,
            uniforms: Uniforms::default(),
        }
    }

    pub fn with_view(mut self) -> Self {
        self.uniforms.camera = Some(CameraUniform::default());
        self
    }

    pub fn with_ambient<C>(mut self, color: C) -> Self
    where
        C: IntoLinear<3>,
    {
        let Linear(color) = color.into_linear();
        self.uniforms.ambient = Some(AmbientUniform::new(color));
        self
    }

    pub fn build<S>(self, handle: LayerHandle<S>) -> Result<GlobalsHandle<S>, ResourceNotFound>
    where
        S: Shader,
    {
        use dunge_shader::View;

        if let View::Camera = S::VIEW {
            assert!(
                self.uniforms.camera.is_some(),
                "the shader requires `view`, but it's not set",
            );
        }

        if S::AMBIENT {
            assert!(
                self.uniforms.ambient.is_some(),
                "the shader requires `ambient`, but it's not set",
            );
        }

        self.resources
            .create_globals(self.render, self.uniforms, handle)
    }
}
