//! Model transformation types and traits.

type Mat = [[f32; 4]; 4];

pub trait IntoMat {
    fn into_mat(self) -> Mat;
}

impl IntoMat for Mat {
    fn into_mat(self) -> Mat {
        self
    }
}

impl IntoMat for glam::Mat4 {
    fn into_mat(self) -> Mat {
        self.to_cols_array_2d()
    }
}

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

impl IntoMat for Position {
    fn into_mat(self) -> Mat {
        Transform {
            pos: self.0,
            ..Default::default()
        }
        .into_mat()
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

impl Transform<Quat> {
    fn mat(self) -> Mat {
        use glam::{Mat4, Quat as Q, Vec3};

        let mat = Mat4::from_scale_rotation_translation(
            Vec3::from(self.scl),
            Q::from_array(self.rot.0),
            Vec3::from(self.pos),
        );

        mat.to_cols_array_2d()
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

impl<Q> IntoMat for Transform<Q>
where
    Q: IntoQuat,
{
    fn into_mat(self) -> Mat {
        Transform {
            pos: self.pos,
            rot: self.rot.into_quat(),
            scl: self.scl,
        }
        .mat()
    }
}

pub trait IntoQuat {
    fn into_quat(self) -> Quat;
}

impl IntoQuat for glam::Quat {
    fn into_quat(self) -> Quat {
        Quat(self.to_array())
    }
}

/// The identity rotation.
#[derive(Clone, Copy, Default)]
pub struct Identity;

impl IntoQuat for Identity {
    fn into_quat(self) -> Quat {
        Quat::default()
    }
}

/// Represents a quaternion.
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

/// The rotation along an axis by an angle.
pub struct AxisAngle(pub [f32; 3], pub f32);

impl IntoQuat for AxisAngle {
    fn into_quat(self) -> Quat {
        use glam::{Quat as Q, Vec3};

        let Self(axis, angle) = self;
        let quat = Q::from_axis_angle(Vec3::from(axis), angle);
        Quat(quat.to_array())
    }
}

/// A type that represents the reversed rotation of the given one.
///
/// # Example
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
