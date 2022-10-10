use {
    crate::layout::{InstanceModel, Plain},
    glam::{Mat4, Quat, Vec3},
    wgpu::{Buffer, Device},
};

/// A data struct for an instance creation.
#[derive(Clone, Copy)]
pub struct InstanceData {
    pub pos: [f32; 3],
    pub rot: [f32; 4],
    pub scl: [f32; 3],
}

impl InstanceData {
    pub(crate) fn to_model(self) -> InstanceModel {
        let pos = self.pos.into();
        let rot = Quat::from_array(self.rot);
        let scl = self.scl.into();

        InstanceModel {
            mat: *Mat4::from_scale_rotation_translation(scl, rot, pos).as_ref(),
        }
    }
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO.into(),
            rot: Quat::IDENTITY.into(),
            scl: Vec3::ONE.into(),
        }
    }
}

pub(crate) struct Instance {
    data: Vec<InstanceData>,
    buffer: Buffer,
}

impl Instance {
    pub fn new(data: Vec<InstanceData>, device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let instances: Vec<_> = data.iter().copied().map(InstanceData::to_model).collect();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: instances.as_slice().as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        Self { data, buffer }
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(crate) fn n_instances(&self) -> u32 {
        u32::try_from(self.data.len()).expect("convert instances len")
    }
}

pub trait IntoInstances {
    fn into_instances(self) -> Vec<InstanceData>;
}

impl IntoInstances for InstanceData {
    fn into_instances(self) -> Vec<InstanceData> {
        vec![self]
    }
}

impl IntoInstances for Vec<InstanceData> {
    fn into_instances(self) -> Vec<InstanceData> {
        self
    }
}

impl IntoInstances for &[InstanceData] {
    fn into_instances(self) -> Vec<InstanceData> {
        Vec::from(self)
    }
}

impl<const N: usize> IntoInstances for [InstanceData; N] {
    fn into_instances(self) -> Vec<InstanceData> {
        Vec::from(self)
    }
}
