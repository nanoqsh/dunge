//! Handles for context's objects management.

use {crate::topology::TriangleList, std::marker::PhantomData};

/// A layer handle. May be obtained from the [`create_layer`](crate::Context::create_layer) method.
#[derive(Clone, Copy)]
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

/// A texture handle. May be obtained from the [`create_texture`](crate::Context::create_texture) method.
#[derive(Clone, Copy)]
#[must_use]
pub struct TextureHandle(pub(crate) u32);

/// An instance handle. May be obtained from the [`create_instances`](crate::Context::create_instances) method.
#[derive(Clone, Copy)]
#[must_use]
pub struct InstanceHandle(pub(crate) u32);

/// A mesh handle. May be obtained from the [`create_mesh`](crate::Context::create_mesh) method.
#[derive(Clone, Copy)]
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

/// A view handle. May be obtained from the [`create_view`](crate::Context::create_view) method.
#[derive(Clone, Copy)]
#[must_use]
pub struct ViewHandle(pub(crate) u32);

/// A light handle. May be obtained from the [`create_light`](crate::Context::create_light) method.
#[derive(Clone, Copy)]
#[must_use]
pub struct LightHandle(pub(crate) u32);

impl LightHandle {
    pub(crate) const DEFAULT: Self = Self(0);
}
