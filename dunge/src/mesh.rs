use {
    crate::{
        buffer::DynamicBufferView,
        topology::{Topology, TriangleList},
        vertex::{self, Vertex},
    },
    std::borrow::Cow,
    wgpu::{Buffer, Device, PrimitiveTopology, Queue},
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
    verts: Buffer,
    indxs: Option<Buffer>,
    vert_size: usize,
    topology: PrimitiveTopology,
}

impl Mesh {
    pub fn new<V, T>(data: &Data<V, T>, device: &Device) -> Self
    where
        V: Vertex,
        T: Topology,
    {
        use {
            std::mem,
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                BufferUsages,
            },
        };

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
            vert_size: mem::size_of::<V>(),
            topology: T::VALUE.into_inner(),
        }
    }

    pub fn update<V, T>(&mut self, data: &Data<V, T>, queue: &Queue) -> Result<(), UpdateError>
    where
        V: Vertex,
        T: Topology,
    {
        use std::mem;

        assert_eq!(self.vert_size, mem::size_of::<V>(), "invalid vertex type");
        assert_eq!(self.topology, T::VALUE.into_inner(), "invalid topology");

        let verts = data.verts;
        if self.verts.size() != verts.len() as u64 {
            return Err(UpdateError::VertexSize);
        }

        if let Some(indxs) = &data.indxs {
            let buf = self.indxs.as_ref().ok_or(UpdateError::Kind)?;
            if buf.size() != indxs.len() as u64 {
                return Err(UpdateError::IndexSize);
            }

            queue.write_buffer(buf, 0, bytemuck::cast_slice(indxs));
        }

        queue.write_buffer(&self.verts, 0, vertex::verts_as_bytes(verts));
        Ok(())
    }

    pub fn vertex_buffer(&self) -> DynamicBufferView {
        DynamicBufferView::new(&self.verts, self.vert_size as u32)
    }

    pub fn index_buffer(&self) -> Option<DynamicBufferView> {
        use std::mem;

        self.indxs
            .as_ref()
            .map(|buf| DynamicBufferView::new(buf, mem::size_of::<u16>() as u32))
    }
}

#[derive(Debug)]
pub enum UpdateError {
    IndexSize,
    VertexSize,
    Kind,
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
