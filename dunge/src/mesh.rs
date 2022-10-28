use {
    crate::{layout::Plain, vertex::Vertex},
    std::borrow::Cow,
    wgpu::{Buffer, Device, Queue},
};

/// A data struct for a mesh creation.
pub struct Data<'a, V> {
    verts: &'a [V],
    indxs: Cow<'a, [[u16; 3]]>,
}

impl<'a, V> Data<'a, V> {
    /// Creates a new [`MeshData`](crate::MeshData) from given vertices and indices.
    ///
    /// Returns `Some` if a data length fits in `u16` and all indices point to the data,
    /// otherwise returns `None`.
    pub fn new(verts: &'a [V], indxs: &'a [[u16; 3]]) -> Option<Self> {
        verts
            .len()
            .try_into()
            .ok()
            .filter(|len| indxs.iter().flatten().all(|i| i < len))
            .map(|_| Self {
                verts,
                indxs: indxs.into(),
            })
    }

    /// Creates a new [`MeshData`](crate::MeshData) from given triangles.
    ///
    /// Returns `Some` if a data length fits in `u16` and is multiple by 3,
    /// otherwise returns `None`.
    pub fn from_triangles(verts: &'a [V]) -> Option<Self> {
        verts
            .len()
            .try_into()
            .ok()
            .filter(|len| len % 3 == 0)
            .map(|len| {
                let indxs = (0..len).step_by(3).map(|i| [i, i + 1, i + 2]).collect();

                Self { verts, indxs }
            })
    }

    /// Creates a new [`MeshData`](crate::MeshData) from given quadrangles.
    ///
    /// Returns `Some` if a data length fits in `u16` and is multiple by 4,
    /// otherwise returns `None`.
    pub fn from_quads(verts: &'a [V]) -> Option<Self> {
        verts
            .len()
            .try_into()
            .ok()
            .filter(|len| len % 4 == 0)
            .map(|len| {
                let indxs = (0..len)
                    .step_by(4)
                    .flat_map(|i| [[i, i + 1, i + 2], [i + 2, i + 1, i + 3]])
                    .collect();

                Self { verts, indxs }
            })
    }
}

impl<'a, V> Clone for Data<'a, V> {
    fn clone(&self) -> Self {
        Self {
            verts: self.verts,
            indxs: self.indxs.clone(),
        }
    }
}

pub(crate) struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    n_indices: u32,
}

impl Mesh {
    pub(crate) fn new<V>(data: &Data<V>, device: &Device) -> Self
    where
        V: Vertex,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: data.verts.as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: data.indxs.as_ref().as_bytes(),
            usage: BufferUsages::INDEX,
        });

        let n_indices = (data.indxs.len() * 3).try_into().expect("too many indexes");

        Self {
            vertex_buffer,
            index_buffer,
            n_indices,
        }
    }

    pub(crate) fn update_data<V>(&mut self, data: &Data<V>, queue: &Queue)
    where
        V: Vertex,
    {
        queue.write_buffer(&self.vertex_buffer, 0, data.verts.as_bytes());
        queue.write_buffer(&self.index_buffer, 0, data.indxs.as_ref().as_bytes());
        self.n_indices = (data.indxs.len() * 3).try_into().expect("too many indexes");
    }

    pub(crate) fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub(crate) fn n_indices(&self) -> u32 {
        self.n_indices
    }
}
