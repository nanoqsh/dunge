use {
    crate::{
        _vertex::Vertex as _Vertex,
        camera::{Camera, Projection, View},
        color::Linear,
        error::{Error, ResourceNotFound, TooManySources, TooManySpaces},
        handles::*,
        mesh::{Data as MeshData, Mesh},
        pipeline::{Parameters as PipelineParameters, Pipeline, VertexLayout},
        render::Render,
        shader_data::{
            Instance, InstanceModel, Light, LightSpace, SourceModel, SpaceData, SpaceModel,
            Texture, TextureData,
        },
        storage::Storage,
        topology::Topology,
        vertex::Vertex,
    },
    dunge_shader::{generate, Scheme, Shader},
};

/// A container of resources for render.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) instances: Storage<Instance>,
    pub(crate) layers: Storage<Pipeline>,
    pub(crate) lights: Storage<Light>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) shaders: Storage<Shader>,
    pub(crate) spaces: Storage<LightSpace>,
    pub(crate) textures: Storage<Texture>,
    pub(crate) views: Storage<Camera>,
}

impl Resources {
    pub fn create_shader<V>(&mut self, scheme: Scheme) -> ShaderHandle<V> {
        let shader = generate(scheme);
        log::debug!("generated shader:\n{}", shader.source);
        let id = self.shaders.insert(shader);
        ShaderHandle::new(id)
    }

    pub fn create_layer<V, T>(
        &mut self,
        render: &Render,
        params: PipelineParameters,
        handle: ShaderHandle<V>,
    ) -> Result<LayerHandle<V, T>, ResourceNotFound>
    where
        V: Vertex,
        T: Topology,
    {
        let vert = VertexLayout::new::<V>();
        let pipeline = Pipeline::new(
            render.context().device(),
            self.shaders.get(handle.id())?,
            &vert,
            PipelineParameters {
                topology: T::VALUE.into_inner(),
                ..params
            },
        );

        let id = self.layers.insert(pipeline);
        Ok(LayerHandle::new(id))
    }

    pub fn _create_layer<V, T>(
        &mut self,
        render: &Render,
        params: PipelineParameters,
    ) -> LayerHandle<V, T>
    where
        V: _Vertex,
        T: Topology,
    {
        let pipeline = render._create_pipeline(
            V::VALUE.into_inner(),
            PipelineParameters {
                topology: T::VALUE.into_inner(),
                ..params
            },
        );

        let id = self.layers.insert(pipeline);
        LayerHandle::new(id)
    }

    pub fn delete_layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<(), ResourceNotFound> {
        self.layers.remove(handle.id())
    }

    pub fn create_texture(&mut self, render: &Render, data: TextureData) -> TextureHandle {
        let texture = Texture::new(
            data,
            render.context().device(),
            render.context().queue(),
            &render.groups().textured,
        );

        let id = self.textures.insert(texture);
        TextureHandle(id)
    }

    pub fn update_texture(
        &self,
        render: &Render,
        handle: TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        let texture = self.textures.get(handle.0)?;
        texture.update_data(data, render.context().queue())?;

        Ok(())
    }

    pub fn delete_texture(&mut self, handle: TextureHandle) -> Result<(), ResourceNotFound> {
        self.textures.remove(handle.0)
    }

    pub fn create_instances(
        &mut self,
        render: &Render,
        models: &[InstanceModel],
    ) -> InstanceHandle {
        let instance = Instance::new(models, render.context().device());
        let id = self.instances.insert(instance);
        InstanceHandle(id)
    }

    pub fn update_instances(
        &self,
        render: &Render,
        handle: InstanceHandle,
        models: &[InstanceModel],
    ) -> Result<(), Error> {
        let instances = self.instances.get(handle.0)?;
        instances.update_models(models, render.context().queue())?;

        Ok(())
    }

    pub fn delete_instances(&mut self, handle: InstanceHandle) -> Result<(), ResourceNotFound> {
        self.instances.remove(handle.0)
    }

    pub fn create_mesh<V, T>(&mut self, render: &Render, data: &MeshData<V, T>) -> MeshHandle<V, T>
    where
        V: Vertex,
        T: Topology,
    {
        let mesh = Mesh::new(data, render.context().device());
        let id = self.meshes.insert(mesh);
        MeshHandle::new(id)
    }

    pub fn _create_mesh<V, T>(&mut self, render: &Render, data: &MeshData<V, T>) -> MeshHandle<V, T>
    where
        V: _Vertex,
        T: Topology,
    {
        let mesh = Mesh::_new(data, render.context().device());
        let id = self.meshes.insert(mesh);
        MeshHandle::new(id)
    }

    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), ResourceNotFound> {
        self.meshes.remove(handle.id())
    }

    pub fn create_view(&mut self, render: &Render, view: View<Projection>) -> ViewHandle {
        let mut camera = Camera::new(render.context().device(), &render.groups().globals);
        camera.set_view(view);
        let id = self.views.insert(camera);
        ViewHandle(id)
    }

    pub fn update_view(
        &mut self,
        handle: ViewHandle,
        view: View<Projection>,
    ) -> Result<(), ResourceNotFound> {
        self.views
            .get_mut(handle.0)
            .map(|camera| camera.set_view(view))
    }

    pub fn delete_view(&mut self, handle: ViewHandle) -> Result<(), ResourceNotFound> {
        self.views.remove(handle.0)
    }

    pub fn create_light(
        &mut self,
        render: &Render,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<LightHandle, TooManySources> {
        let light = Light::new(
            ambient.0,
            srcs,
            render.context().device(),
            &render.groups().lights,
        )?;

        let id = self.lights.insert(light);
        Ok(LightHandle(id))
    }

    pub fn update_light(
        &mut self,
        render: &Render,
        handle: LightHandle,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<(), Error> {
        let light = self.lights.get_mut(handle.0)?;
        light.update_sources(ambient.0, srcs, render.context().queue())?;

        Ok(())
    }

    pub fn update_nth_light(
        &self,
        render: &Render,
        handle: LightHandle,
        n: usize,
        source: SourceModel,
    ) -> Result<(), Error> {
        let light = self.lights.get(handle.0)?;
        light.update_nth(n, source, render.context().queue())?;

        Ok(())
    }

    pub fn delete_light(&mut self, handle: LightHandle) -> Result<(), ResourceNotFound> {
        self.lights.remove(handle.0)
    }

    pub fn create_space(
        &mut self,
        render: &Render,
        spaces: &[SpaceModel],
        data: &[SpaceData],
    ) -> Result<SpaceHandle, TooManySpaces> {
        let ls = LightSpace::new(
            spaces,
            data,
            render.context().device(),
            render.context().queue(),
            &render.groups().space,
        )?;

        let id = self.spaces.insert(ls);
        Ok(SpaceHandle(id))
    }

    pub fn update_space(
        &mut self,
        render: &Render,
        handle: SpaceHandle,
        spaces: &[SpaceModel],
        data: &[SpaceData],
    ) -> Result<(), Error> {
        let ls = self.spaces.get_mut(handle.0)?;
        ls.update_spaces(spaces, data, render.context().queue())?;

        Ok(())
    }

    pub fn update_nth_space(
        &self,
        render: &Render,
        handle: SpaceHandle,
        n: usize,
        space: SpaceModel,
    ) -> Result<(), Error> {
        let ls = self.spaces.get(handle.0)?;
        ls.update_nth_space(n, space, render.context().queue())?;

        Ok(())
    }

    pub fn update_nth_space_color(
        &self,
        render: &Render,
        handle: SpaceHandle,
        n: usize,
        color: Linear<f32, 3>,
    ) -> Result<(), Error> {
        let ls = self.spaces.get(handle.0)?;
        ls.update_nth_color(n, color.0, render.context().queue())?;

        Ok(())
    }

    pub fn update_nth_space_data(
        &self,
        render: &Render,
        handle: SpaceHandle,
        n: usize,
        data: SpaceData,
    ) -> Result<(), Error> {
        let ls = self.spaces.get(handle.0)?;
        ls.update_nth_data(n, data, render.context().queue())?;

        Ok(())
    }

    pub fn delete_space(&mut self, handle: SpaceHandle) -> Result<(), ResourceNotFound> {
        self.spaces.remove(handle.0)
    }
}
