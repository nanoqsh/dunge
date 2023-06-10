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

/// An instance handle. May be obtained from the [`create_instances`](crate::Context::create_instances) method.
#[must_use]
#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);

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

/// A light handle. May be obtained from the [`create_light`](crate::Context::create_light) method.
#[must_use]
#[derive(Clone, Copy)]
pub struct LightHandle(pub(crate) u32);

impl LightHandle {
    pub(crate) const DEFAULT: Self = Self(0);
}

/// A mesh handle. May be obtained from the [`create_mesh`](crate::Context::create_mesh) method.
#[must_use]
pub struct MeshHandle<V, T = TriangleList>(u32, PhantomData<(V, T)>);

impl<V, T> MeshHandle<V, T> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    pub(crate) fn id(self) -> u32 {
        self.0
    }
}

impl<V, T> Clone for MeshHandle<V, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, T> Copy for MeshHandle<V, T> {}

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

/// A space handle. May be obtained from the [`create_space`](crate::Context::create_space) method.
#[must_use]
#[derive(Clone, Copy)]
pub struct SpaceHandle(pub(crate) u32);

impl SpaceHandle {
    pub(crate) const DEFAULT: Self = Self(0);
}

/// A texture handle. May be obtained from the [`create_texture`](crate::Context::create_texture) method.
#[must_use]
#[derive(Clone, Copy)]
pub struct _TextureHandle(pub(crate) u32);

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

/// A view handle. May be obtained from the [`create_view`](crate::Context::create_view) method.
#[must_use]
#[derive(Clone, Copy)]
pub struct _ViewHandle(pub(crate) u32);
