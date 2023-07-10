use {
    crate::{
        buffer::BufferView,
        render::State,
        topology::{Topology, TriangleList},
        vertex::{self, Vertex},
    },
    std::{borrow::Cow, marker::PhantomData, sync::Arc},
    wgpu::{Buffer, Queue},
};

/// A data struct for a mesh creation.
#[derive(Clone)]
pub struct Data<'a, V, T = TriangleList>
where
    T: Topology,
{
    verts: &'a [V],
    indxs: Option<Cow<'a, [T::Face]>>,
}

impl<'a, V, T> Data<'a, V, T>
where
    T: Topology,
{
    /// Creates a new [`MeshData`](crate::MeshData) from given vertices.
    pub fn from_verts(verts: &'a [V]) -> Self {
        Self { verts, indxs: None }
    }
}

impl<'a, V> Data<'a, V> {
    /// Creates a new [`MeshData`](crate::MeshData) from given vertices and indices.
    ///
    /// # Errors
    /// Will return
    /// - [`MeshError::TooManyVertices`](crate::error::MeshError::TooManyVertices)
    ///   if vertices length doesn't fit in `u16`.
    /// - [`MeshError::WrongIndex`](crate::error::MeshError::WrongIndex)
    ///   if the vertex index is out of bounds of the vertex slice.
    pub fn new(verts: &'a [V], indxs: &'a [[u16; 3]]) -> Result<Self, Error> {
        let len: u16 = verts.len().try_into().map_err(|_| Error::TooManyVertices)?;

        if indxs.iter().flatten().any(|&i| i >= len) {
            return Err(Error::WrongIndex);
        }

        Ok(Self {
            verts,
            indxs: Some(Cow::Borrowed(indxs)),
        })
    }

    /// Creates a new [`MeshData`](crate::MeshData) from given quadrangles.
    ///
    /// # Errors
    /// Will return
    /// - [`MeshError::TooManyVertices`](crate::error::MeshError::TooManyVertices)
    ///   if vertices length doesn't fit in `u16`.
    pub fn from_quads(verts: &'a [[V; 4]]) -> Result<Self, Error> {
        use std::slice;

        let new_len = verts.len() * 4;
        let len: u16 = new_len.try_into().map_err(|_| Error::TooManyVertices)?;

        Ok(Self {
            verts: unsafe { slice::from_raw_parts(verts.as_ptr().cast(), new_len) },
            indxs: Some(
                (0..len)
                    .step_by(4)
                    .flat_map(|i| [[i, i + 1, i + 2], [i + 2, i + 1, i + 3]])
                    .collect(),
            ),
        })
    }
}

/// An error returned from the [`MeshData`](crate::MeshData) constructors.
#[derive(Debug)]
pub enum Error {
    /// Vertices length doesn't fit in `u16`.
    TooManyVertices,

    /// The vertex index is out of bounds of the vertex slice.
    WrongIndex,
}

pub struct Mesh<V, T = TriangleList> {
    verts: Buffer,
    indxs: Option<Buffer>,
    queue: Arc<Queue>,
    ty: PhantomData<(V, T)>,
}

impl<V, T> Mesh<V, T> {
    pub(crate) fn new(data: &Data<V, T>, state: &State) -> Self
    where
        V: Vertex,
        T: Topology,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let device = state.device();
        Self {
            verts: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: vertex::verts_as_bytes(data.verts),
                usage: BufferUsages::VERTEX,
            }),
            indxs: data.indxs.as_deref().map(|indxs| {
                device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("index buffer"),
                    contents: bytemuck::cast_slice(indxs),
                    usage: BufferUsages::INDEX,
                })
            }),
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    /// Updates the mesh with a new [data](`Data`).
    ///
    /// # Errors
    /// Will return
    /// - [`MeshUpdateError::VertexSize`](crate::error::MeshUpdateError::VertexSize)
    ///   if a slice of vertices is passed with a wrong length.
    /// - [`MeshUpdateError::IndexSize`](crate::error::MeshUpdateError::IndexSize)
    ///   if a slice of indices is passed with a wrong length.
    /// - [`MeshUpdateError::NoIndices`](crate::error::MeshUpdateError::NoIndices)
    ///   if no indices are passed but they are required.
    pub fn update(&mut self, data: &Data<V, T>) -> Result<(), UpdateError>
    where
        V: Vertex,
        T: Topology,
    {
        let verts = data.verts;
        if self.verts.size() != verts.len() as u64 {
            return Err(UpdateError::VertexSize);
        }

        if let Some(indxs) = &data.indxs {
            let buf = self.indxs.as_ref().ok_or(UpdateError::NoIndices)?;
            if buf.size() != indxs.len() as u64 {
                return Err(UpdateError::IndexSize);
            }

            self.queue.write_buffer(buf, 0, bytemuck::cast_slice(indxs));
        }

        self.queue
            .write_buffer(&self.verts, 0, vertex::verts_as_bytes(verts));

        Ok(())
    }

    pub(crate) fn buffer(&self) -> MeshBuffer {
        MeshBuffer {
            verts: BufferView::new::<V>(&self.verts),
            indxs: self.indxs.as_ref().map(BufferView::new::<u16>),
        }
    }
}

/// An error returned from the [`update`](Mesh::update) function.
#[derive(Debug)]
pub enum UpdateError {
    /// A slice of vertices is passed with a wrong length.
    VertexSize,

    /// A slice of indices is passed with a wrong length.
    IndexSize,

    /// No indices are passed but they are required.
    NoIndices,
}

pub(crate) struct MeshBuffer<'a> {
    pub verts: BufferView<'a>,
    pub indxs: Option<BufferView<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_quads() {
        let verts = [[0, 1, 2, 3], [4, 5, 6, 7]];
        let data = Data::from_quads(&verts).expect("mesh data");
        let indxs = data.indxs.expect("indices");
        assert_eq!(data.verts.len(), 8);
        assert_eq!(indxs.len(), 4);
        assert_eq!([data.verts[0], data.verts[1], data.verts[2]], indxs[0]);
        assert_eq!([data.verts[2], data.verts[1], data.verts[3]], indxs[1]);
        assert_eq!([data.verts[4], data.verts[5], data.verts[6]], indxs[2]);
        assert_eq!([data.verts[6], data.verts[5], data.verts[7]], indxs[3]);
    }
}
