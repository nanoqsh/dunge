//! Handles for context's objects management.

use std::marker::PhantomData;

/// A layer handle. May be obtained from the [`create_layer`](crate::Context::create_layer) method.
#[derive(Clone, Copy)]
pub struct LayerHandle<V>(u32, PhantomData<V>);

impl<V> LayerHandle<V> {
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
pub struct MeshHandle<V>(u32, PhantomData<V>);

impl<V> MeshHandle<V> {
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
