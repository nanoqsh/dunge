//! Handles for context's objects management.

use std::marker::PhantomData;

#[must_use]
pub struct LightsHandle<S>(u32, PhantomData<S>);

impl<S> LightsHandle<S> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<S> Clone for LightsHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for LightsHandle<S> {}

#[must_use]
pub struct SpacesHandle<S>(u32, PhantomData<S>);

impl<S> SpacesHandle<S> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<S> Clone for SpacesHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for SpacesHandle<S> {}
