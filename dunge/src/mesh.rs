//! The mesh and mesh data types.

use {
    crate::{state::State, vertex, Vertex},
    std::{borrow::Cow, error, fmt, marker::PhantomData},
    wgpu::{Buffer, RenderPass},
};

type Face = [u16; 3];

#[derive(Clone)]
pub struct MeshData<'a, V> {
    verts: &'a [V],
    indxs: Option<Cow<'a, [Face]>>,
}

impl<'a, V> MeshData<'a, V> {
    /// Creates a [mesh data](crate::mesh::MeshData) from given vertices.
    pub const fn from_verts(verts: &'a [V]) -> Self {
        Self { verts, indxs: None }
    }

    /// Creates a [mesh data](crate::mesh::MeshData) from given vertices and indices.
    ///
    /// # Errors
    /// Returns an [error](crate::mesh::Error) if the passed data is incorrect.
    pub fn new(verts: &'a [V], indxs: &'a [Face]) -> Result<Self, Error> {
        let len: u16 = verts.len().try_into().map_err(|_| Error::TooManyVertices)?;
        if let Some(index) = indxs.iter().flatten().copied().find(|&i| i >= len) {
            return Err(Error::InvalidIndex { index });
        }

        let indxs = Some(Cow::Borrowed(indxs));
        Ok(Self { verts, indxs })
    }

    /// Creates a [mesh data](crate::mesh::MeshData) from given quadrilaterals.
    ///
    /// # Errors
    /// Returns an [error](crate::mesh::TooManyVertices) if too many vertices are passed.
    pub fn from_quads(verts: &'a [[V; 4]]) -> Result<Self, TooManyVertices> {
        let verts = verts.as_flattened();
        let indxs = {
            let len = u16::try_from(verts.len()).map_err(|_| TooManyVertices)?;
            let faces = (0..len)
                .step_by(4)
                .flat_map(|i| [[i, i + 1, i + 2], [i, i + 2, i + 3]])
                .collect();

            Some(faces)
        };

        Ok(Self { verts, indxs })
    }
}

/// An error returned from the [mesh data](crate::mesh::MeshData) constructors.
#[derive(Debug)]
pub enum Error {
    /// Vertices length doesn't fit in [`u16`](std::u16) integer.
    TooManyVertices,

    /// The vertex index is out of bounds of the vertex slice.
    InvalidIndex { index: u16 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TooManyVertices => write!(f, "too many vertices"),
            Self::InvalidIndex { index } => write!(f, "invalid index: {index}"),
        }
    }
}

impl error::Error for Error {}

/// Vertices length doesn't fit in [`u16`](std::u16) integer.
#[derive(Debug)]
pub struct TooManyVertices;

impl fmt::Display for TooManyVertices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "too many vertices")
    }
}

impl error::Error for TooManyVertices {}

pub struct Mesh<V> {
    verts: Buffer,
    indxs: Option<Buffer>,
    ty: PhantomData<V>,
}

impl<V> Mesh<V> {
    pub(crate) fn new(state: &State, data: &MeshData<V>) -> Self
    where
        V: Vertex,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let device = state.device();
        let verts = {
            let desc = BufferInitDescriptor {
                label: None,
                contents: vertex::verts_as_bytes(data.verts),
                usage: BufferUsages::VERTEX,
            };

            device.create_buffer_init(&desc)
        };

        let indxs = data.indxs.as_deref().map(|indxs| {
            let desc = BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indxs),
                usage: BufferUsages::INDEX,
            };

            device.create_buffer_init(&desc)
        });

        Self {
            verts,
            indxs,
            ty: PhantomData,
        }
    }

    pub(crate) fn is_indexed(&self) -> bool {
        self.indxs.is_some()
    }

    pub(crate) fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, slot: u32, count: u32) {
        use wgpu::IndexFormat;

        pass.set_vertex_buffer(slot, self.verts.slice(..));
        match &self.indxs {
            Some(indxs) => {
                pass.set_index_buffer(indxs.slice(..), IndexFormat::Uint16);
                let len = indxs.size() as u32 / size_of::<u16>() as u32;
                pass.draw_indexed(0..len, 0, 0..count);
            }
            None => {
                let len = self.verts.size() as u32 / size_of::<V>() as u32;
                pass.draw(0..len, 0..count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_quads() {
        let verts = [[0, 1, 2, 3], [4, 5, 6, 7]];
        let data = MeshData::from_quads(&verts).expect("mesh data");
        let indxs = data.indxs.expect("indices");
        assert_eq!(data.verts.len(), 8);
        assert_eq!(indxs.len(), 4);
        assert_eq!([data.verts[0], data.verts[1], data.verts[2]], indxs[0]);
        assert_eq!([data.verts[0], data.verts[2], data.verts[3]], indxs[1]);
        assert_eq!([data.verts[4], data.verts[5], data.verts[6]], indxs[2]);
        assert_eq!([data.verts[4], data.verts[6], data.verts[7]], indxs[3]);
    }
}
