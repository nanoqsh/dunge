use {
    crate::layout::{InstanceModel, Plain},
    wgpu::{Buffer, Device},
};

/// A data struct for an instance creation.
#[derive(Clone, Copy)]
pub struct InstanceData<R> {
    pub pos: [f32; 3],
    pub rot: R,
    pub scl: [f32; 3],
}

impl<R> InstanceData<R>
where
    R: Rotation,
{
    pub(crate) fn into_model(self) -> InstanceModel {
        let [xt, yt, zt] = self.pos;
        let [x_axis, y_axis, z_axis] = self.rot.into_rotation_mat();
        let [xs, ys, zs] = self.scl;

        let [xx, xy, xz] = x_axis.map(|v| v * xs);
        let [yx, yy, yz] = y_axis.map(|v| v * ys);
        let [zx, zy, zz] = z_axis.map(|v| v * zs);

        InstanceModel {
            mat: [
                [xx, xy, xz, 0.],
                [yx, yy, yz, 0.],
                [zx, zy, zz, 0.],
                [xt, yt, zt, 1.],
            ],
        }
    }
}

impl Default for InstanceData<Quat> {
    fn default() -> Self {
        Self {
            pos: [0., 0., 0.],
            rot: Quat([0., 0., 0., 1.]),
            scl: [1., 1., 1.],
        }
    }
}

pub(crate) struct Instance {
    models: Vec<InstanceModel>,
    buffer: Buffer,
}

impl Instance {
    pub fn new(models: Vec<InstanceModel>, device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: models.as_slice().as_bytes(),
            usage: BufferUsages::VERTEX,
        });

        Self { models, buffer }
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(crate) fn n_instances(&self) -> u32 {
        u32::try_from(self.models.len()).expect("convert instances len")
    }
}

pub trait Rotation {
    fn into_rotation_mat(self) -> [[f32; 3]; 3];
}

pub struct Quat(pub [f32; 4]);

impl Rotation for Quat {
    fn into_rotation_mat(self) -> [[f32; 3]; 3] {
        let Self([x, y, z, w]) = self;

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;
        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        [
            [1. - yy - zz, xy + wz, xz - wy],
            [xy - wz, 1. - xx - zz, yz + wx],
            [xz + wy, yz - wx, 1. - xx - yy],
        ]
    }
}

pub struct AxisAngle(pub [f32; 3], pub f32);

impl Rotation for AxisAngle {
    fn into_rotation_mat(self) -> [[f32; 3]; 3] {
        let Self(axis, angle) = self;

        let (sin, cos) = angle.sin_cos();
        let [xsin, ysin, zsin] = axis.map(|v| v * sin);
        let [x, y, z] = axis;
        let [xs, ys, zs] = axis.map(f32::sqrt);
        let omc = 1. - cos;
        let xyomc = x * y * omc;
        let xzomc = x * z * omc;
        let yzomc = y * z * omc;

        [
            [xs * omc + cos, xyomc + zsin, xzomc - ysin],
            [xyomc - zsin, ys * omc + cos, yzomc + xsin],
            [xzomc + ysin, yzomc - xsin, zs * omc + cos],
        ]
    }
}
