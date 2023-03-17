//! Handles for context's objects management.

use {crate::topology::TriangleList, std::marker::PhantomData};

/// A layer handle. May be obtained from the [`create_layer`](crate::Context::create_layer) method.
#[derive(Clone, Copy)]
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
pub struct TextureHandle(pub(crate) u32);

/// An instance handle. May be obtained from the [`create_instances`](crate::Context::create_instances) method.
#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);

/// A mesh handle. May be obtained from the [`create_mesh`](crate::Context::create_mesh) method.
#[derive(Clone, Copy)]
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
pub struct ViewHandle(pub(crate) u32);
