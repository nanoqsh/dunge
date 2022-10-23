use {
    crate::{layout::Plain, vertex::Vertex},
    std::borrow::Cow,
    wgpu::{Buffer, Device, Queue},
};

/// A data struct for a mesh creation.
pub struct MeshData<'a, V> {
    verts: &'a [V],
    indxs: Cow<'a, [[u16; 3]]>,
}

impl<'a, V> MeshData<'a, V> {
    /// Creates a new `MeshData` from given vertices and indices.
    ///
    /// Returns `Some` if a data length fits in `u16` and all indices point to the data,
    /// otherwise returns `None`.
    pub fn new(verts: &'a [V], indxs: &'a [[u16; 3]]) -> Option<Self> {
        (u16::try_from(verts.len()).is_ok()
            && indxs
                .iter()
                .flatten()
                .all(|&i| usize::from(i) < verts.len()))
        .then(|| Self {
            verts,
            indxs: indxs.into(),
        })
    }

    /// Creates a new `MeshData` from given triangles.
    ///
    /// Returns `Some` if a data length fits in `u16` and is multiple by 3,
    /// otherwise returns `None`.
    pub fn from_triangles(verts: &'a [V]) -> Option<Self> {
        (u16::try_from(verts.len()).is_ok() && verts.len() % 3 == 0).then(|| {
            let indxs = (0..verts.len() as u16)
                .step_by(3)
                .map(|i| [i, i + 1, i + 2])
                .collect();

            Self { verts, indxs }
        })
    }

    /// Creates a new `MeshData` from given quadrangles.
    ///
    /// Returns `Some` if a data length fits in `u16` and is multiple by 4,
    /// otherwise returns `None`.
    pub fn from_quads(verts: &'a [V]) -> Option<Self> {
        (u16::try_from(verts.len()).is_ok() && verts.len() % 4 == 0).then(|| {
            let indxs = (0..verts.len() as u16)
                .step_by(4)
                .flat_map(|i| [[i, i + 1, i + 2], [i + 2, i + 1, i + 3]])
                .collect();

            Self { verts, indxs }
        })
    }
}

impl<'a, V> Clone for MeshData<'a, V> {
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
    pub(crate) fn new<V>(data: MeshData<V>, device: &Device) -> Self
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

    pub(crate) fn update_data<V>(&mut self, data: MeshData<V>, queue: &Queue)
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
