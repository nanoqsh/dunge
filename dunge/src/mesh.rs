use {
    crate::{
        topology::{Topology, TriangleList},
        vertex::{self, Vertex},
    },
    std::borrow::Cow,
    wgpu::{Buffer, Device},
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
    /// See [`MeshError`](crate::MeshError) for detailed info.
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
    /// See [`MeshError`](crate::MeshError) for detailed info.
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

#[derive(Debug)]
pub enum Error {
    /// Returns when the vertices length too big and doesn't fit in `u16`.
    TooManyVertices,

    /// Returns when the vertex index is out of bounds of the vertex slice.
    WrongIndex,
}

pub(crate) struct Mesh {
    buf: Buffer,
    kind: Kind,
}

impl Mesh {
    pub fn new<V, T>(data: &Data<V, T>, device: &Device) -> Self
    where
        V: Vertex,
        T: Topology,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self {
            buf: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: vertex::verts_as_bytes(data.verts),
                usage: BufferUsages::VERTEX,
            }),
            kind: match &data.indxs {
                Some(indxs) => Kind::indexed(bytemuck::cast_slice(indxs), device),
                None => Kind::sequential(data.verts.len()),
            },
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.buf
    }

    pub fn kind(&self) -> &Kind {
        &self.kind
    }
}

pub(crate) enum Kind {
    Indexed { buf: Buffer, len: u32 },
    Sequential { len: u32 },
}

impl Kind {
    fn indexed(indxs: &[u16], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self::Indexed {
            buf: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(indxs),
                usage: BufferUsages::INDEX,
            }),
            len: indxs.len().try_into().expect("too many indexes"),
        }
    }

    fn sequential(len: usize) -> Self {
        Self::Sequential {
            len: len.try_into().expect("too many vertices"),
        }
    }
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
