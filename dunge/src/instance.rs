use std::convert::TryInto;

use {
    crate::layout::{InstanceModel, Plain},
    wgpu::{Buffer, Device, Queue},
};

/// A data struct for an instance creation.
#[derive(Clone, Copy)]
pub struct InstanceData<R = Identity> {
    pub pos: [f32; 3],
    pub rot: R,
    pub scl: [f32; 3],
}

impl<R> InstanceData<R>
where
    R: Rotation,
{
    pub(crate) fn into_model(self) -> InstanceModel {
        fn rotation(Quat([x, y, z, w]): Quat) -> [[f32; 3]; 3] {
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

        let [xt, yt, zt] = self.pos;
        let [x_axis, y_axis, z_axis] = rotation(self.rot.into_quat());
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

impl Default for InstanceData<Identity> {
    fn default() -> Self {
        Self {
            pos: [0., 0., 0.],
            rot: Identity,
            scl: [1., 1., 1.],
        }
    }
}

pub(crate) struct Instance {
    buffer: Buffer,
    n_instances: u32,
}

impl Instance {
    pub(crate) fn new(models: &[InstanceModel], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: models.as_bytes(),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let n_instances = models.len().try_into().expect("convert instances len");

        Self {
            buffer,
            n_instances,
        }
    }

    pub(crate) fn update_models(&mut self, models: &[InstanceModel], queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, models.as_bytes());
        self.n_instances = models.len().try_into().expect("convert instances len");
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(crate) fn n_instances(&self) -> u32 {
        self.n_instances
    }
}

pub trait Rotation {
    fn into_quat(self) -> Quat;
}

#[derive(Default)]
pub struct Identity;

impl Rotation for Identity {
    fn into_quat(self) -> Quat {
        Quat::default()
    }
}

pub struct Quat(pub [f32; 4]);

impl Default for Quat {
    fn default() -> Self {
        Self([0., 0., 0., 1.])
    }
}

impl Rotation for Quat {
    fn into_quat(self) -> Self {
        self
    }
}

pub struct AxisAngle(pub [f32; 3], pub f32);

impl Rotation for AxisAngle {
    fn into_quat(self) -> Quat {
        let Self(axis, angle) = self;

        let (sin, cos) = (angle * 0.5).sin_cos();
        let [x, y, z] = axis.map(|v| v * sin);

        Quat([x, y, z, cos])
    }
}

#[derive(Default)]
pub struct Inversed<R>(pub R);

impl<R> Rotation for Inversed<R>
where
    R: Rotation,
{
    fn into_quat(self) -> Quat {
        let Quat([x, y, z, w]) = self.0.into_quat();
        Quat([-x, -y, -z, w])
    }
}
