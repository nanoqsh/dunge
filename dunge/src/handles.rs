//! Handles for context's objects management.

use {crate::topology::TriangleList, std::marker::PhantomData};

#[must_use]
pub struct GlobalsHandle<S>(u32, PhantomData<S>);

impl<S> GlobalsHandle<S> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<S> Clone for GlobalsHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for GlobalsHandle<S> {}

/// A layer handle. May be obtained from the [`create_layer`](crate::Context::create_layer) method.
#[must_use]
pub struct LayerHandle<V, T = TriangleList>(u32, PhantomData<(V, T)>);

impl<V, T> LayerHandle<V, T> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<V, T> Clone for LayerHandle<V, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, T> Copy for LayerHandle<V, T> {}

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

/// A shader handle. May be obtained from the [`create_shader`](crate::Context::create_shader) method.
#[must_use]
pub struct ShaderHandle<S>(u32, PhantomData<S>);

impl<S> ShaderHandle<S> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<S> Clone for ShaderHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for ShaderHandle<S> {}

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

#[must_use]
pub struct TexturesHandle<S>(u32, PhantomData<S>);

impl<S> TexturesHandle<S> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<S> Clone for TexturesHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for TexturesHandle<S> {}
