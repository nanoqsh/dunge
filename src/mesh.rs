use {
    crate::{
        frame::Instance,
        layout::{Layout, Plain},
    },
    glam::{Quat, Vec3},
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
    instances: Vec<Instance>,
    instance_buffer: Buffer,
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

        const NUM_INSTANCES: usize = 10;

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

        let instances: Vec<_> = (0..NUM_INSTANCES)
            .map(|n| {
                use std::f32::consts::TAU;

                let seg = TAU / NUM_INSTANCES as f32;
                let rad = 4.;
                let pos = Vec3 {
                    x: (n as f32 * seg).sin() * rad,
                    y: 0.,
                    z: (n as f32 * seg).cos() * rad,
                };
                let rot = Quat::from_axis_angle(Vec3::Z, n as f32 * seg);

                Instance {
                    pos,
                    rot,
                    scl: Vec3::ONE,
                }
            })
            .collect();

        let instance_data: Vec<_> = instances.iter().map(Instance::to_model).collect();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: instance_data.as_slice().as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            n_indices,
            instances,
            instance_buffer,
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

    pub(crate) fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }

    pub(crate) fn n_instances(&self) -> u32 {
        u32::try_from(self.instances.len()).expect("convert len")
    }
}
