use {
    crate::{
        mesh::Mesh,
        render::{MeshHandle, Render, TextureHandle},
        texture::Texture,
        Error,
    },
    ahash::AHashMap as Map,
    wgpu::RenderPass,
};

/// A struct represented a current frame
/// and exists during a frame render.
pub struct Frame<'d> {
    pub(crate) pass: RenderPass<'d>,
    pub(crate) resources: &'d Resources,
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
        self.pass
            .set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint16);
        self.pass.draw_indexed(0..mesh.n_indices(), 0, 0..1);

        Ok(())
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) textures: Storage<Texture>,
}

pub(crate) struct Storage<T> {
    map: Map<u32, T>,
    counter: u32,
}

impl<T> Storage<T> {
    pub(crate) fn insert(&mut self, value: T) -> u32 {
        let index = self.counter;
        self.counter = self.counter.wrapping_add(1);
        self.map.insert(index, value);
        index
    }

    pub(crate) fn get(&self, index: u32) -> Result<&T, Error> {
        self.map.get(&index).ok_or(Error::ResourceNotFound)
    }

    pub(crate) fn remove(&mut self, index: u32) {
        self.map.remove(&index);
    }
}

impl<T> Default for Storage<T> {
    fn default() -> Self {
        Self {
            map: Map::default(),
            counter: 0,
        }
    }
}
