use {
    crate::{
        layout::InstanceModel,
        mesh::Mesh,
        render::{MeshHandle, Render, TextureHandle},
        storage::Storage,
        texture::Texture,
        Error,
    },
    glam::{Mat4, Quat, Vec3},
    wgpu::RenderPass,
};

/// A struct represented a current frame
/// and exists during a frame render.
pub struct Frame<'d> {
    pub(crate) resources: &'d Resources,
    pub(crate) pass: RenderPass<'d>,
}

impl Frame<'_> {
    pub fn bind_texture(&mut self, TextureHandle(id): TextureHandle) -> Result<(), Error> {
        let texture = self.resources.textures.get(id)?;
        self.pass
            .set_bind_group(Render::TEXTURE_BIND_GROUP, texture.bind_group(), &[]);

        Ok(())
    }

    pub fn draw_mesh(&mut self, MeshHandle(id): MeshHandle) -> Result<(), Error> {
        use wgpu::IndexFormat;

        let mesh = self.resources.meshes.get(id)?;
        self.pass
            .set_vertex_buffer(Render::VERTEX_BUFFER_SLOT, mesh.vertex_buffer().slice(..));
        self.pass.set_vertex_buffer(
            Render::INSTANCE_BUFFER_SLOT,
            mesh.instance_buffer().slice(..),
        );
        self.pass
            .set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
        self.pass
            .draw_indexed(0..mesh.n_indices(), 0, 0..mesh.n_instances());

        Ok(())
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) textures: Storage<Texture>,
}

pub(crate) struct Instance {
    pub(crate) pos: Vec3,
    pub(crate) rot: Quat,
    pub(crate) scl: Vec3,
}

impl Instance {
    pub(crate) fn to_model(&self) -> InstanceModel {
        InstanceModel {
            mat: *Mat4::from_scale_rotation_translation(self.scl, self.rot, self.pos).as_ref(),
        }
    }
}
