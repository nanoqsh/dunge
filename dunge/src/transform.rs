//! Model transformation types and traits.

use crate::layout::InstanceModel;

/// An instance position.
///
/// Descibes the position in X, Y, Z coordinates.
#[derive(Clone, Copy, Default)]
pub struct Position(pub [f32; 3]);

impl From<[f32; 3]> for Position {
    fn from(v: [f32; 3]) -> Self {
        Self(v)
    }
}

/// An instance transform.
///
/// It descrides position, rotation and scale.
/// The rotation may be any type that implements [`IntoQuat`].
#[derive(Clone, Copy)]
pub struct Transform<Q = Identity> {
    /// The position in X, Y, Z coordinates.
    pub pos: [f32; 3],

    /// The rotation is one of an [`IntoQuat`] type.
    pub rot: Q,

    /// The scale in X, Y, Z axes.
    pub scl: [f32; 3],
}

impl<Q> Transform<Q>
where
    Q: IntoQuat,
{
    pub(crate) fn into_model(self) -> InstanceModel {
        let Self { pos, rot, scl } = self;
        let transform = Transform {
            pos,
            rot: rot.into_quat(),
            scl,
        };
        transform.into_model_quat()
    }
}

impl Transform<Quat> {
    fn into_model_quat(self) -> InstanceModel {
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
        let [x_axis, y_axis, z_axis] = rotation(self.rot);
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

impl Default for Transform<Identity> {
    fn default() -> Self {
        Self {
            pos: [0., 0., 0.],
            rot: Identity,
            scl: [1., 1., 1.],
        }
    }
}

pub trait IntoTransform {
    type IntoQuat;
    fn into_transform(self) -> Transform<Self::IntoQuat>;
}

impl IntoTransform for Position {
    type IntoQuat = Identity;

    fn into_transform(self) -> Transform<Self::IntoQuat> {
        Transform {
            pos: self.0,
            ..Default::default()
        }
    }
}

impl<Q> IntoTransform for Transform<Q> {
    type IntoQuat = Q;

    fn into_transform(self) -> Transform<Self::IntoQuat> {
        self
    }
}

pub trait IntoQuat {
    fn into_quat(self) -> Quat;
}

#[derive(Default)]
pub struct Identity;

impl IntoQuat for Identity {
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

impl IntoQuat for Quat {
    fn into_quat(self) -> Self {
        self
    }
}

pub struct AxisAngle(pub [f32; 3], pub f32);

impl IntoQuat for AxisAngle {
    fn into_quat(self) -> Quat {
        let Self(axis, angle) = self;

        let (sin, cos) = (angle * 0.5).sin_cos();
        let [x, y, z] = axis.map(|v| v * sin);

        Quat([x, y, z, cos])
    }
}

/// A type that represents the reversed rotation of the given one.
///
/// ## Example
/// ```
/// # use dunge::transform::{AxisAngle, ReverseRotation};
/// # let n = 1.;
/// // The rotation along Y by `n` radians.
/// let axis = AxisAngle([0., 1., 0.], n);
/// // Now it's reversed. The rotation along Y by `-n` radians.
/// let back = ReverseRotation(axis);
/// ```
#[derive(Default)]
pub struct ReverseRotation<Q>(pub Q);

impl<Q> IntoQuat for ReverseRotation<Q>
where
    Q: IntoQuat,
{
    fn into_quat(self) -> Quat {
        let Quat([x, y, z, w]) = self.0.into_quat();
        Quat([-x, -y, -z, w])
    }
}
