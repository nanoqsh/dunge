use crate::{
    error::{SourceError, SpaceError},
    handles::*,
    layer::Layer,
    render::State,
    shader_data::{
        lights::{Lights, Parameters as LightsParameters, Variables as LightsVariables},
        spaces::{Parameters as SpacesParameters, Spaces, Variables as SpacesVariables},
        Source, SpaceData,
    },
    storage::Storage,
};

/// A container of resources for render.
#[derive(Default)]
pub(crate) struct Resources {
    pub lights: Storage<Lights>,
    pub spaces: Storage<Spaces>,
}

impl Resources {
    pub fn create_lights<S, T>(
        &mut self,
        state: &State,
        variables: LightsVariables,
        layer: &Layer<S, T>,
    ) -> LightsHandle<S> {
        let lights = layer.pipeline().lights().expect("the shader has no lights");
        let params = LightsParameters {
            variables,
            bindings: &lights.bindings,
            layout: &lights.layout,
        };

        let lights = Lights::new(params, state);
        let id = self.lights.insert(lights);
        LightsHandle::new(id)
    }

    pub fn update_lights_sources<S>(
        &mut self,
        handle: LightsHandle<S>,
        index: usize,
        sources: &[Source],
    ) -> Result<(), SourceError> {
        self.lights
            .get_mut(handle.id())?
            .update_array(index, 0, sources)?;

        Ok(())
    }

    pub fn create_spaces<S, T>(
        &mut self,
        state: &State,
        variables: SpacesVariables,
        layer: &Layer<S, T>,
    ) -> SpacesHandle<S> {
        let spaces = layer.pipeline().spaces().expect("the shader has no spaces");
        let params = SpacesParameters {
            variables,
            bindings: &spaces.bindings,
            layout: &spaces.layout,
        };

        let spaces = Spaces::new(params, state);
        let id = self.spaces.insert(spaces);
        SpacesHandle::new(id)
    }

    pub fn update_spaces_data<S>(
        &self,
        handle: SpacesHandle<S>,
        index: usize,
        data: SpaceData,
    ) -> Result<(), SpaceError> {
        self.spaces.get(handle.id())?.update_data(index, data)?;
        Ok(())
    }
}
