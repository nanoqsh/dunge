use {
    crate::vertex::Vertex,
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
        V: Vertex,
    {
        use {
            std::{mem, slice},
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                BufferUsages,
            },
        };

        Self {
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: unsafe {
                    slice::from_raw_parts(
                        data.verts.as_ptr().cast(),
                        data.verts.len() * mem::size_of::<V>(),
                    )
                },
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("index buffer"),
                contents: unsafe {
                    slice::from_raw_parts(
                        data.indxs.as_ptr().cast(),
                        data.indxs.len() * mem::size_of::<[u16; 3]>(),
                    )
                },
                usage: BufferUsages::INDEX,
            }),
            n_indices: u32::try_from(data.indxs.len() * 3).expect("too many indexes"),
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
