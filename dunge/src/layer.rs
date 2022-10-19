use {
    crate::{
        camera::Camera,
        instance::Instance,
        mesh::Mesh,
        r#loop::Error,
        render::ViewHandle,
        render::{InstanceHandle, MeshHandle, TextureHandle},
        shader_consts,
        storage::Storage,
        texture::Texture,
        vertex::TextureVertex,
    },
    std::marker::PhantomData,
    wgpu::{Queue, RenderPass},
};

pub struct Layer<'l, V> {
    pass: RenderPass<'l>,
    size: (u32, u32),
    queue: &'l Queue,
    resources: &'l Resources,
    instance: Option<&'l Instance>,
    vertex_type: PhantomData<V>,
}

impl<'l, V> Layer<'l, V> {
    pub(crate) fn new(
        pass: RenderPass<'l>,
        size: (u32, u32),
        queue: &'l Queue,
        resources: &'l Resources,
    ) -> Self {
        Self {
            pass,
            size,
            queue,
            resources,
            instance: None,
            vertex_type: PhantomData,
        }
    }

    pub fn bind_instance(&mut self, handle: InstanceHandle) -> Result<(), Error> {
        let instance = self.resources.instances.get(handle.0)?;
        self.instance = Some(instance);

        Ok(())
    }

    pub fn bind_view(&mut self, handle: ViewHandle) -> Result<(), Error> {
        const CAMERA_GROUP: u32 = {
            assert!(shader_consts::textured::CAMERA.group == shader_consts::color::CAMERA.group);
            shader_consts::textured::CAMERA.group
        };

        let camera = self.resources.views.get(handle.0)?;
        camera.resize(self.size, self.queue);

        self.pass
            .set_bind_group(CAMERA_GROUP, camera.bind_group(), &[]);

        Ok(())
    }

    pub fn draw(&mut self, handle: MeshHandle<V>) -> Result<(), Error> {
        use wgpu::IndexFormat;

        let mesh = self.resources.meshes.get(handle.id())?;

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
}

impl Layer<'_, TextureVertex> {
    pub fn bind_texture(&mut self, handle: TextureHandle) -> Result<(), Error> {
        let texture = self.resources.textures.get(handle.0)?;
        self.pass.set_bind_group(
            shader_consts::textured::S_DIFFUSE.group,
            texture.bind_group(),
            &[],
        );

        Ok(())
    }
}

/// A container of drawable resources.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) textures: Storage<Texture>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) views: Storage<Camera>,
}
