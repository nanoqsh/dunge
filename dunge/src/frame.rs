use {
    crate::{
        instance::Instance,
        mesh::Mesh,
        pipline::Pipeline,
        r#loop::Error,
        render::{InstanceHandle, MeshHandle, TextureHandle},
        shader_consts,
        storage::Storage,
        texture::Texture,
        vertex::VertexType,
    },
    wgpu::{BindGroup, RenderPass},
};

/// A struct represented a current frame
/// and exists during a frame render.
pub struct Frame<'d> {
    pub(crate) main_pipeline: &'d MainPipeline,
    pub(crate) camera_bind_group: &'d BindGroup,
    pub(crate) resources: &'d Resources,
    pub(crate) pass: RenderPass<'d>,
    instance: Option<&'d Instance>,
    current_vertex_type: VertexType,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(
        main_pipeline: &'d MainPipeline,
        camera_bind_group: &'d BindGroup,
        resources: &'d Resources,
        pass: RenderPass<'d>,
    ) -> Self {
        const DEFAULT_PIPELINE: VertexType = VertexType::Texture;

        let mut frame = Self {
            main_pipeline,
            camera_bind_group,
            resources,
            pass,
            instance: None,
            current_vertex_type: DEFAULT_PIPELINE,
        };

        frame.set_vertex_type(DEFAULT_PIPELINE);
        frame
    }

    pub fn bind_texture(&mut self, TextureHandle(id): TextureHandle) -> Result<(), Error> {
        if self.current_vertex_type != VertexType::Texture {
            return Err(Error::TextureBindingUnavailable);
        }

        let texture = self.resources.textures.get(id)?;
        self.pass.set_bind_group(
            shader_consts::textured::S_DIFFUSE.group,
            texture.bind_group(),
            &[],
        );

        Ok(())
    }

    pub fn set_instance(&mut self, InstanceHandle(id): InstanceHandle) -> Result<(), Error> {
        let instance = self.resources.instances.get(id)?;
        self.instance = Some(instance);

        Ok(())
    }

    pub fn draw_mesh(&mut self, MeshHandle(id): MeshHandle) -> Result<(), Error> {
        use wgpu::IndexFormat;

        let mesh = self.resources.meshes.get(id)?;

        if mesh.vertex_type() != self.current_vertex_type {
            self.set_vertex_type(mesh.vertex_type());
        }

        let n_instances = match self.instance {
            Some(instance) => {
                self.pass.set_vertex_buffer(
                    shader_consts::INSTANCE_BUFFER_SLOT,
                    instance.buffer().slice(..),
                );

                instance.n_instances()
            }
            None => return Err(Error::InstanceNotSet),
        };

        self.pass.set_vertex_buffer(
            shader_consts::VERTEX_BUFFER_SLOT,
            mesh.vertex_buffer().slice(..),
        );

        self.pass
            .set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);

        self.pass
            .draw_indexed(0..mesh.n_indices(), 0, 0..n_instances);

        Ok(())
    }

    fn set_vertex_type(&mut self, ty: VertexType) {
        match ty {
            VertexType::Texture => {
                self.pass.set_pipeline(self.main_pipeline.textured.as_ref());
                self.pass.set_bind_group(
                    shader_consts::textured::CAMERA.group,
                    self.camera_bind_group,
                    &[],
                );
            }
            VertexType::Color => {
                self.pass.set_pipeline(self.main_pipeline.color.as_ref());
                self.pass.set_bind_group(
                    shader_consts::color::CAMERA.group,
                    self.camera_bind_group,
                    &[],
                );
            }
        }

        self.current_vertex_type = ty;
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
}

pub(crate) struct MainPipeline {
    pub(crate) textured: Pipeline,
    pub(crate) color: Pipeline,
}
