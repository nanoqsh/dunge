use crate::{
    camera::{Camera, View},
    error::{ResourceNotFound, SourceError, SpaceError, TexturesError},
    handles::*,
    layer::Layer,
    render::Render,
    shader_data::{
        globals::{Globals, Parameters as GlobalsParameters, Variables as GlobalsVariables},
        lights::{Lights, Parameters as LightsParameters, Variables as LightsVariables},
        spaces::{Parameters as SpacesParameters, Spaces, Variables as SpacesVariables},
        textures::{Parameters as TexturesParameters, Textures, Variables as TexturesVariables},
        Source, SpaceData, TextureData,
    },
    storage::Storage,
};

/// A container of resources for render.
#[derive(Default)]
pub(crate) struct Resources {
    pub globals: Storage<Globals>,
    pub lights: Storage<Lights>,
    pub spaces: Storage<Spaces>,
    pub textures: Storage<Textures>,
}

impl Resources {
    pub fn create_globals<S, T>(
        &mut self,
        render: &Render,
        variables: GlobalsVariables,
        layer: &Layer<S, T>,
    ) -> GlobalsHandle<S> {
        let globals = layer
            .pipeline()
            .globals()
            .expect("the shader has no globals");

        let params = GlobalsParameters {
            camera: Camera::default(),
            variables,
            bindings: &globals.bindings,
            layout: &globals.layout,
        };

        let globals = Globals::new(params, render.state().device());
        let id = self.globals.insert(globals);
        GlobalsHandle::new(id)
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
            .write_ambient(col, render.state().queue());

        Ok(())
    }

    pub fn create_textures<S, T>(
        &mut self,
        render: &Render,
        variables: TexturesVariables,
        layer: &Layer<S, T>,
    ) -> TexturesHandle<S> {
        let textures = layer
            .pipeline()
            .textures()
            .expect("the shader has no textures");

        let params = TexturesParameters {
            variables,
            bindings: &textures.bindings,
            layout: &textures.layout,
        };

        let context = render.state();
        let textures = Textures::new(params, context.device(), context.queue());
        let id = self.textures.insert(textures);
        TexturesHandle::new(id)
    }

    pub fn update_textures_map<S>(
        &self,
        render: &Render,
        handle: TexturesHandle<S>,
        data: TextureData,
    ) -> Result<(), TexturesError> {
        self.textures
            .get(handle.id())?
            .update_data(data, render.state().queue())?;

        Ok(())
    }

    pub fn create_lights<S, T>(
        &mut self,
        render: &Render,
        variables: LightsVariables,
        layer: &Layer<S, T>,
    ) -> LightsHandle<S> {
        let lights = layer.pipeline().lights().expect("the shader has no lights");
        let params = LightsParameters {
            variables,
            bindings: &lights.bindings,
            layout: &lights.layout,
        };

        let lights = Lights::new(params, render.state().device());
        let id = self.lights.insert(lights);
        LightsHandle::new(id)
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
            render.state().queue(),
        )?;

        Ok(())
    }

    pub fn create_spaces<S, T>(
        &mut self,
        render: &Render,
        variables: SpacesVariables,
        layer: &Layer<S, T>,
    ) -> SpacesHandle<S> {
        let spaces = layer.pipeline().spaces().expect("the shader has no spaces");
        let params = SpacesParameters {
            variables,
            bindings: &spaces.bindings,
            layout: &spaces.layout,
        };

        let context = render.state();
        let spaces = Spaces::new(params, context.device(), context.queue());
        let id = self.spaces.insert(spaces);
        SpacesHandle::new(id)
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
            .update_data(index, data, render.state().queue())?;

        Ok(())
    }
}
