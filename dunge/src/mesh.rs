use {
    crate::{
        layout::Plain,
        topology::{Topology, TriangleList},
        vertex::Vertex,
    },
    std::{borrow::Cow, slice},
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
    /// Returns `Some` if a data length fits in `u16` and all indices point to the data,
    /// otherwise returns `None`.
    pub fn new(verts: &'a [V], indxs: &'a [[u16; 3]]) -> Option<Self> {
        let len: u16 = verts.len().try_into().ok()?;
        indxs.iter().flatten().all(|&i| i < len).then_some(Self {
            verts,
            indxs: Some(indxs.into()),
        })
    }

    /// Creates a new [`MeshData`](crate::MeshData) from given quadrangles.
    ///
    /// Returns `Some` if a data length fits in `u16` and is multiple by 4,
    /// otherwise returns `None`.
    pub fn from_quads(verts: &'a [V]) -> Option<Self> {
        let len: u16 = verts.len().try_into().ok()?;
        (len % 4 == 0).then_some({
            Self {
                verts,
                indxs: Some(
                    (0..len)
                        .step_by(4)
                        .flat_map(|i| [[i, i + 1, i + 2], [i + 2, i + 1, i + 3]])
                        .collect(),
                ),
            }
        })
    }
}

pub(crate) struct Mesh {
    vertex_buffer: Buffer,
    ty: Type,
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
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: data.verts.as_bytes(),
                usage: BufferUsages::VERTEX,
            }),
            ty: match &data.indxs {
                Some(indxs) => Type::indexed(
                    unsafe { slice::from_raw_parts(indxs.as_ptr().cast(), indxs.len() * 3) },
                    device,
                ),
                None => Type::sequential(data.verts),
            },
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn mesh_type(&self) -> &Type {
        &self.ty
    }
}

pub(crate) enum Type {
    Indexed {
        index_buffer: Buffer,
        n_indices: u32,
    },
    Sequential {
        n_vertices: u32,
    },
}

impl Type {
    fn indexed(indxs: &[u16], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self::Indexed {
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("index buffer"),
                contents: indxs.as_ref().as_bytes(),
                usage: BufferUsages::INDEX,
            }),
            n_indices: indxs.len().try_into().expect("too many indexes"),
        }
    }

    fn sequential<V>(verts: &[V]) -> Self {
        Self::Sequential {
            n_vertices: verts.len().try_into().expect("too many vertices"),
        }
    }
}
