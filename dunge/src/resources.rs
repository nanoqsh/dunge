use {
    crate::{
        _vertex::_Vertex,
        camera::{Camera, Projection, View, _Camera},
        color::Linear,
        error::{
            Error, ResourceNotFound, SourceError, TexturesError, TooManySources, TooManySpaces,
        },
        handles::*,
        mesh::{Data as MeshData, Mesh},
        pipeline::{Parameters as PipelineParameters, Pipeline, VertexLayout},
        render::Render,
        shader::Shader,
        shader_data::{
            globals::{Globals, Parameters as GlobalsParameters, Variables as GlobalsVariables},
            lights::{Lights, Parameters as LightsParameters, Variables as LightsVariables},
            textures::{
                Parameters as TexturesParameters, Textures, Variables as TexturesVariables,
            },
            Instance, InstanceModel, Light, LightSpace, Source, SourceModel, SpaceData, SpaceModel,
            Texture, TextureData,
        },
        storage::Storage,
        topology::Topology,
        vertex::Vertex,
    },
    dunge_shader::{Scheme, Shader as ShaderData},
};

/// A container of resources for render.
#[derive(Default)]
pub(crate) struct Resources {
    pub(crate) globals: Storage<Globals>,
    pub(crate) instances: Storage<Instance>,
    pub(crate) layers: Storage<Pipeline>,
    pub(crate) _lights: Storage<Light>,
    pub(crate) lights: Storage<Lights>,
    pub(crate) meshes: Storage<Mesh>,
    pub(crate) shaders: Storage<ShaderData>,
    pub(crate) spaces: Storage<LightSpace>,
    pub(crate) _textures: Storage<Texture>,
    pub(crate) textures: Storage<Textures>,
    pub(crate) views: Storage<_Camera>,
}

impl Resources {
    pub fn create_globals<S>(
        &mut self,
        render: &Render,
        variables: GlobalsVariables,
        handle: LayerHandle<S>,
    ) -> Result<GlobalsHandle<S>, ResourceNotFound> {
        let layer = self.layers.get(handle.id())?;
        let globals = layer.globals().expect("the shader has no globals");
        let params = GlobalsParameters {
            camera: Camera::new(),
            variables,
            bindings: &globals.bindings,
            layout: &globals.layout,
        };

        let globals = Globals::new(params, render.context().device());
        let id = self.globals.insert(globals);
        Ok(GlobalsHandle::new(id))
    }

    pub fn update_globals_view<S>(
        &mut self,
        handle: GlobalsHandle<S>,
        view: View<Projection>,
    ) -> Result<(), ResourceNotFound> {
        self.globals.get_mut(handle.id())?.set_view(view);
        Ok(())
    }

    pub fn update_globals_ambient<S>(
        &self,
        render: &Render,
        handle: GlobalsHandle<S>,
        color: Linear<f32, 3>,
    ) -> Result<(), ResourceNotFound> {
        self.globals
            .get(handle.id())?
            .write_ambient(color.0, render.context().queue());

        Ok(())
    }

    pub fn create_textures<S>(
        &mut self,
        render: &Render,
        variables: TexturesVariables,
        handle: LayerHandle<S>,
    ) -> Result<TexturesHandle<S>, ResourceNotFound> {
        let layer = self.layers.get(handle.id())?;
        let textures = layer.textures().expect("the shader has no textures");
        let params = TexturesParameters {
            variables,
            bindings: &textures.bindings,
            layout: &textures.layout,
        };

        let context = render.context();
        let textures = Textures::new(params, context.device(), context.queue());
        let id = self.textures.insert(textures);
        Ok(TexturesHandle::new(id))
    }

    pub fn update_textures_map<S>(
        &self,
        render: &Render,
        handle: TexturesHandle<S>,
        data: TextureData,
    ) -> Result<(), TexturesError> {
        self.textures
            .get(handle.id())?
            .update_data(data, render.context().queue())?;

        Ok(())
    }

    pub fn create_lights<S>(
        &mut self,
        render: &Render,
        variables: LightsVariables,
        handle: LayerHandle<S>,
    ) -> Result<LightsHandle<S>, ResourceNotFound> {
        let layer = self.layers.get(handle.id())?;
        let lights = layer.lights().expect("the shader has no lights");
        let params = LightsParameters {
            variables,
            bindings: &lights.bindings,
            layout: &lights.layout,
        };

        let lights = Lights::new(params, render.context().device());
        let id = self.lights.insert(lights);
        Ok(LightsHandle::new(id))
    }

    pub fn update_lights_sources<S>(
        &mut self,
        render: &Render,
        handle: LightsHandle<S>,
        index: usize,
        offset: usize,
        sources: &[Source],
    ) -> Result<(), SourceError> {
        self.lights.get_mut(handle.id())?.update_array(
            index,
            offset,
            sources,
            render.context().queue(),
        )?;

        Ok(())
    }

    pub fn create_shader<S>(&mut self, scheme: Scheme) -> ShaderHandle<S> {
        let shader = ShaderData::generate(scheme);
        log::debug!("generated shader:\n{}", shader.source);
        let id = self.shaders.insert(shader);
        ShaderHandle::new(id)
    }

    pub fn create_layer<S, T>(
        &mut self,
        render: &Render,
        params: PipelineParameters,
        handle: ShaderHandle<S>,
    ) -> Result<LayerHandle<S, T>, ResourceNotFound>
    where
        S: Shader,
        T: Topology,
    {
        let vert = VertexLayout::new::<S::Vertex>();
        let pipeline = Pipeline::new(
            render.context().device(),
            self.shaders.get(handle.id())?,
            Some(&vert),
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

    pub fn create_texture(&mut self, render: &Render, data: TextureData) -> _TextureHandle {
        let texture = Texture::new(
            data,
            render.context().device(),
            render.context().queue(),
            &render._groups().textured,
        );

        let id = self._textures.insert(texture);
        _TextureHandle(id)
    }

    pub fn update_texture(
        &self,
        render: &Render,
        handle: _TextureHandle,
        data: TextureData,
    ) -> Result<(), Error> {
        let texture = self._textures.get(handle.0)?;
        texture.update_data(data, render.context().queue())?;

        Ok(())
    }

    pub fn delete_texture(&mut self, handle: _TextureHandle) -> Result<(), ResourceNotFound> {
        self._textures.remove(handle.0)
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

    pub fn create_view(&mut self, render: &Render, view: View<Projection>) -> _ViewHandle {
        let mut camera = _Camera::new(render.context().device(), &render._groups().globals);
        camera.set_view(view);
        let id = self.views.insert(camera);
        _ViewHandle(id)
    }

    pub fn update_view(
        &mut self,
        handle: _ViewHandle,
        view: View<Projection>,
    ) -> Result<(), ResourceNotFound> {
        self.views
            .get_mut(handle.0)
            .map(|camera| camera.set_view(view))
    }

    pub fn delete_view(&mut self, handle: _ViewHandle) -> Result<(), ResourceNotFound> {
        self.views.remove(handle.0)
    }

    pub fn create_light(
        &mut self,
        render: &Render,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<_LightHandle, TooManySources> {
        let light = Light::new(
            ambient.0,
            srcs,
            render.context().device(),
            &render._groups().lights,
        )?;

        let id = self._lights.insert(light);
        Ok(_LightHandle(id))
    }

    pub fn update_light(
        &mut self,
        render: &Render,
        handle: _LightHandle,
        ambient: Linear<f32, 3>,
        srcs: &[SourceModel],
    ) -> Result<(), Error> {
        let light = self._lights.get_mut(handle.0)?;
        light.update_sources(ambient.0, srcs, render.context().queue())?;

        Ok(())
    }

    pub fn update_nth_light(
        &self,
        render: &Render,
        handle: _LightHandle,
        n: usize,
        source: SourceModel,
    ) -> Result<(), Error> {
        let light = self._lights.get(handle.0)?;
        light.update_nth(n, source, render.context().queue())?;

        Ok(())
    }

    pub fn delete_light(&mut self, handle: _LightHandle) -> Result<(), ResourceNotFound> {
        self._lights.remove(handle.0)
    }

    pub fn create_space(
        &mut self,
        render: &Render,
        spaces: &[SpaceModel],
        data: &[SpaceData],
    ) -> Result<_SpaceHandle, TooManySpaces> {
        let ls = LightSpace::new(
            spaces,
            data,
            render.context().device(),
            render.context().queue(),
            &render._groups().space,
        )?;

        let id = self.spaces.insert(ls);
        Ok(_SpaceHandle(id))
    }

    pub fn update_space(
        &mut self,
        render: &Render,
        handle: _SpaceHandle,
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
        handle: _SpaceHandle,
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
        handle: _SpaceHandle,
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
        handle: _SpaceHandle,
        n: usize,
        data: SpaceData,
    ) -> Result<(), Error> {
        let ls = self.spaces.get(handle.0)?;
        ls.update_nth_data(n, data, render.context().queue())?;

        Ok(())
    }

    pub fn delete_space(&mut self, handle: _SpaceHandle) -> Result<(), ResourceNotFound> {
        self.spaces.remove(handle.0)
    }
}
