use {
    crate::{
        camera::{Camera, View},
        color::{Color, Rgb},
        layer::Layer,
        pipeline::Globals as Bindings,
        render::State,
        shader::{Shader, ShaderInfo},
        shader_data::{ambient::AmbientUniform, Model},
    },
    std::{marker::PhantomData, sync::Arc},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Queue},
};

pub struct Globals<S> {
    group: u32,
    bind_group: BindGroup,
    camera: Option<(Camera, Buffer)>,
    ambient: Option<Buffer>,
    queue: Arc<Queue>,
    ty: PhantomData<S>,
}

impl<S> Globals<S> {
    fn new(params: Parameters, state: &State) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BindGroupDescriptor, BindGroupEntry, BufferUsages,
        };

        let Parameters {
            bindings,
            variables,
            layout,
            ..
        } = params;

        let device = state.device();
        let camera = variables.view.map(|view| {
            let model = Model::default();
            let buf = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[model]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let mut camera = Camera::default();
            camera.update_view(view);
            (camera, buf)
        });

        let ambient = variables.ambient.map(|uniform| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        });

        let entries: Vec<_> = [
            camera.as_ref().map(|(_, buf)| BindGroupEntry {
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
            group: bindings.group,
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("globals bind group"),
            }),
            camera,
            ambient,
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    pub fn update_view(&mut self, view: View)
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        assert!(info.has_camera, "the shader has no view");

        let (camera, _) = self.camera.as_mut().expect("camera");
        camera.update_view(view);
    }

    pub fn update_ambient(&self, Color(col): Rgb)
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        assert!(info.has_ambient, "the shader has no ambient");

        let buf = self.ambient.as_ref().expect("ambient");
        let uniform = AmbientUniform::new(col);
        self.queue
            .write_buffer(buf, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub(crate) fn update_size(&self, size: (u32, u32)) {
        let Some((camera, buf)) = &self.camera else {
            return;
        };

        let uniform = camera.model(size);
        self.queue
            .write_buffer(buf, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub(crate) fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

struct Parameters<'a> {
    variables: Variables,
    bindings: &'a Bindings,
    layout: &'a BindGroupLayout,
}

#[derive(Default)]
struct Variables {
    view: Option<View>,
    ambient: Option<AmbientUniform>,
}

#[must_use]
pub struct Builder<'a> {
    state: &'a State,
    variables: Variables,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self {
            state,
            variables: Variables::default(),
        }
    }

    pub fn with_view(mut self, view: View) -> Self {
        self.variables.view = Some(view);
        self
    }

    pub fn with_ambient(mut self, Color(col): Rgb) -> Self {
        self.variables.ambient = Some(AmbientUniform::new(col));
        self
    }

    #[must_use]
    pub fn build<S, T>(self, layer: &Layer<S, T>) -> Globals<S>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        if info.has_camera {
            assert!(
                self.variables.view.is_some(),
                "the shader requires `view`, but it's not set",
            );
        }

        if info.has_ambient {
            assert!(
                self.variables.ambient.is_some(),
                "the shader requires `ambient`, but it's not set",
            );
        }

        let globals = layer
            .pipeline()
            .globals()
            .expect("the shader has no globals");

        let params = Parameters {
            variables: self.variables,
            bindings: &globals.bindings,
            layout: &globals.layout,
        };

        Globals::new(params, self.state)
    }
}
