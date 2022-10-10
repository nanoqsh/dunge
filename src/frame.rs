use {
    crate::{
        instance::Instance,
        mesh::Mesh,
        render::{InstanceHandle, MeshHandle, Render, TextureHandle},
        storage::Storage,
        texture::Texture,
        Error,
    },
    wgpu::RenderPass,
};

/// A struct represented a current frame
/// and exists during a frame render.
pub struct Frame<'d> {
    pub(crate) resources: &'d Resources,
    pub(crate) pass: RenderPass<'d>,
    instance: Option<&'d Instance>,
}

impl<'d> Frame<'d> {
    pub(crate) fn new(resources: &'d Resources, pass: RenderPass<'d>) -> Self {
        Self {
            resources,
            pass,
            instance: None,
        }
    }

    pub fn bind_texture(&mut self, TextureHandle(id): TextureHandle) -> Result<(), Error> {
        let texture = self.resources.textures.get(id)?;
        self.pass
            .set_bind_group(Render::TEXTURE_BIND_GROUP, texture.bind_group(), &[]);

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

        let n_instances = match self.instance {
            Some(instance) => {
                self.pass
                    .set_vertex_buffer(Render::INSTANCE_BUFFER_SLOT, instance.buffer().slice(..));

                instance.n_instances()
            }
            None => return Err(Error::InstanceNotSet),
        };

        self.pass
            .set_vertex_buffer(Render::VERTEX_BUFFER_SLOT, mesh.vertex_buffer().slice(..));

        self.pass
            .set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);

        self.pass
            .draw_indexed(0..mesh.n_indices(), 0, 0..n_instances);

        Ok(())
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
}
