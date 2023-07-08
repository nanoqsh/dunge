use {
    crate::{
        camera::{Camera, View},
        color::{Color, Rgb},
        handles::GlobalsHandle,
        layer::Layer,
        pipeline::Globals as Bindings,
        render::State,
        resources::Resources,
        shader::{Shader, ShaderInfo},
        shader_data::{ambient::AmbientUniform, Model},
    },
    std::sync::Arc,
    wgpu::{BindGroup, BindGroupLayout, Buffer, Queue},
};

pub(crate) struct Globals {
    group: u32,
    bind_group: BindGroup,
    camera: Option<(Camera, Buffer)>,
    ambient: Option<Buffer>,
    queue: Arc<Queue>,
}

impl Globals {
    pub fn new(params: Parameters, state: &State) -> Self {
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
        let camera = variables.camera.map(|uniform| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
        });

        let ambient = variables.ambient.map(|uniform| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("ambient buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
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
            group: bindings.group,
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries: &entries,
                label: Some("globals bind group"),
            }),
            camera: camera.map(|buf| (params.camera, buf)),
            ambient,
            queue: Arc::clone(state.queue()),
        }
    }

    pub fn set_view(&mut self, view: View) {
        let (camera, _) = self.camera.as_mut().expect("camera");
        camera.set_view(view);
    }

    pub fn write_camera(&self, size: (u32, u32)) {
        let Some((camera, buf)) = &self.camera else {
            return;
        };

        let uniform = camera.model(size);
        self.queue
            .write_buffer(buf, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn write_ambient(&self, color: [f32; 3]) {
        let Some(buf) = &self.ambient else {
            return;
        };

        let uniform = AmbientUniform::new(color);
        self.queue
            .write_buffer(buf, 0, bytemuck::cast_slice(&[uniform]));
    }

    pub fn bind(&self) -> (u32, &BindGroup) {
        (self.group, &self.bind_group)
    }
}

pub(crate) struct Parameters<'a> {
    pub camera: Camera,
    pub variables: Variables,
    pub bindings: &'a Bindings,
    pub layout: &'a BindGroupLayout,
}

#[derive(Default)]
pub(crate) struct Variables {
    pub camera: Option<Model>,
    pub ambient: Option<AmbientUniform>,
}

pub struct Builder<'a> {
    resources: &'a mut Resources,
    state: &'a State,
    variables: Variables,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(resources: &'a mut Resources, state: &'a State) -> Self {
        Self {
            resources,
            state,
            variables: Variables::default(),
        }
    }

    pub fn with_view(mut self) -> Self {
        self.variables.camera = Some(Model::default());
        self
    }

    pub fn with_ambient(mut self, Color(col): Rgb) -> Self {
        self.variables.ambient = Some(AmbientUniform::new(col));
        self
    }

    pub fn build<S, T>(self, layer: &Layer<S, T>) -> GlobalsHandle<S>
    where
        S: Shader,
    {
        let info = ShaderInfo::new::<S>();
        if info.has_camera {
            assert!(
                self.variables.camera.is_some(),
                "the shader requires `view`, but it's not set",
            );
        }

        if info.has_ambient {
            assert!(
                self.variables.ambient.is_some(),
                "the shader requires `ambient`, but it's not set",
            );
        }

        self.resources
            .create_globals(self.state, self.variables, layer)
    }
}
