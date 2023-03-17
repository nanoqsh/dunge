use std::marker::PhantomData;

/// A layer handle.
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

/// A texture handle.
#[derive(Clone, Copy)]
pub struct TextureHandle(pub(crate) u32);

/// A mesh handle.
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

/// An instance handle.
#[derive(Clone, Copy)]
pub struct InstanceHandle(pub(crate) u32);

/// A view handle.
#[derive(Clone, Copy)]
pub struct ViewHandle(pub(crate) u32);
