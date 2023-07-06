use {
    crate::{
        camera::{Camera, View},
        error::{Error, ResourceNotFound, SourceError, SpaceError, TexturesError},
        handles::*,
        mesh::{Data as MeshData, Mesh},
        pipeline::{Parameters as PipelineParameters, Pipeline, VertexLayout},
        render::Render,
        shader::Shader,
        shader_data::{
            globals::{Globals, Parameters as GlobalsParameters, Variables as GlobalsVariables},
            lights::{Lights, Parameters as LightsParameters, Variables as LightsVariables},
            spaces::{Parameters as SpacesParameters, Spaces, Variables as SpacesVariables},
            textures::{
                Parameters as TexturesParameters, Textures, Variables as TexturesVariables,
            },
            Instance, Model, Source, SpaceData, TextureData,
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
    pub globals: Storage<Globals>,
    pub instances: Storage<Instance>,
    pub layers: Storage<Pipeline>,
    pub lights: Storage<Lights>,
    pub meshes: Storage<Mesh>,
    pub shaders: Storage<ShaderData>,
    pub spaces: Storage<Spaces>,
    pub textures: Storage<Textures>,
}

impl Resources {
    pub fn create_globals<S, T>(
        &mut self,
        render: &Render,
        variables: GlobalsVariables,
        handle: LayerHandle<S, T>,
    ) -> Result<GlobalsHandle<S>, ResourceNotFound> {
        let layer = self.layers.get(handle.id())?;
        let globals = layer.globals().expect("the shader has no globals");
        let params = GlobalsParameters {
            camera: Camera::default(),
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
        view: View,
    ) -> Result<(), ResourceNotFound> {
        self.globals.get_mut(handle.id())?.set_view(view);
        Ok(())
    }

    pub fn update_globals_ambient<S>(
        &self,
        render: &Render,
        handle: GlobalsHandle<S>,
        col: [f32; 3],
    ) -> Result<(), ResourceNotFound> {
        self.globals
            .get(handle.id())?
            .write_ambient(col, render.context().queue());

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
        sources: &[Source],
    ) -> Result<(), SourceError> {
        self.lights.get_mut(handle.id())?.update_array(
            index,
            0,
            sources,
            render.context().queue(),
        )?;

        Ok(())
    }

    pub fn create_spaces<S>(
        &mut self,
        render: &Render,
        variables: SpacesVariables,
        handle: LayerHandle<S>,
    ) -> Result<SpacesHandle<S>, ResourceNotFound> {
        let layer = self.layers.get(handle.id())?;
        let spaces = layer.spaces().expect("the shader has no spaces");
        let params = SpacesParameters {
            variables,
            bindings: &spaces.bindings,
            layout: &spaces.layout,
        };

        let context = render.context();
        let spaces = Spaces::new(params, context.device(), context.queue());
        let id = self.spaces.insert(spaces);
        Ok(SpacesHandle::new(id))
    }

    pub fn update_spaces_data<S>(
        &self,
        render: &Render,
        handle: SpacesHandle<S>,
        index: usize,
        data: SpaceData,
    ) -> Result<(), SpaceError> {
        self.spaces
            .get(handle.id())?
            .update_data(index, data, render.context().queue())?;

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

    pub fn delete_layer<V, T>(
        &mut self,
        handle: LayerHandle<V, T>,
    ) -> Result<(), ResourceNotFound> {
        self.layers.remove(handle.id())
    }

    pub fn create_instances(&mut self, render: &Render, models: &[Model]) -> InstanceHandle {
        let instance = Instance::new(models, render.context().device());
        let id = self.instances.insert(instance);
        InstanceHandle(id)
    }

    pub fn update_instances(
        &self,
        render: &Render,
        handle: InstanceHandle,
        models: &[Model],
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

    pub fn delete_mesh<V, T>(&mut self, handle: MeshHandle<V, T>) -> Result<(), ResourceNotFound> {
        self.meshes.remove(handle.id())
    }
}
