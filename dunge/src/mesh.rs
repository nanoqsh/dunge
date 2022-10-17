use {
    crate::{
        layout::Plain,
        vertex::{Vertex, VertexType},
    },
    wgpu::{Buffer, Device, Queue},
};

/// A data struct for a mesh creation.
#[derive(Clone, Copy)]
pub struct MeshData<'a, V> {
    verts: &'a [V],
    indxs: &'a [[u16; 3]],
}

impl<'a, V> MeshData<'a, V> {
    /// Creates a new `MeshData`.
    ///
    /// Returns `Some` if a data length fits in `u16` and all indices point to the data,
    /// otherwise returns `None`.
    pub fn new(verts: &'a [V], indxs: &'a [[u16; 3]]) -> Option<Self> {
        if verts.len() <= usize::from(u16::MAX)
            && indxs
                .iter()
                .flatten()
                .all(|&i| usize::from(i) < verts.len())
        {
            Some(Self { verts, indxs })
        } else {
            None
        }
    }
}

pub(crate) struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    n_indices: u32,
    pipeline: VertexType,
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
            contents: data.indxs.as_bytes(),
            usage: BufferUsages::INDEX,
        });

        let n_indices = (data.indxs.len() * 3).try_into().expect("too many indexes");

        Self {
            vertex_buffer,
            index_buffer,
            n_indices,
            pipeline: V::TYPE,
        }
    }

    pub(crate) fn update_data<V>(&mut self, data: MeshData<V>, queue: &Queue)
    where
        V: Vertex,
    {
        queue.write_buffer(&self.vertex_buffer, 0, data.verts.as_bytes());
        queue.write_buffer(&self.index_buffer, 0, data.indxs.as_bytes());
        self.n_indices = (data.indxs.len() * 3).try_into().expect("too many indexes");
        self.pipeline = V::TYPE;
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

    pub(crate) fn pipeline(&self) -> VertexType {
        self.pipeline
    }
}
