use {
    crate::layout::{Layout, Plain},
    wgpu::{Buffer, Device},
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
            && indxs.iter().all(|&[a, b, c]| {
                usize::from(a) <= verts.len()
                    && usize::from(b) <= verts.len()
                    && usize::from(c) <= verts.len()
            })
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
}

impl Mesh {
    pub(crate) fn new<V>(data: MeshData<V>, device: &Device) -> Self
    where
        V: Layout,
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

        let n_indices = u32::try_from(data.indxs.len() * 3).expect("too many indexes");

        Self {
            vertex_buffer,
            index_buffer,
            n_indices,
        }
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
