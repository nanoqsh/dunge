//! Model transformation types and traits.

use {
    crate::shader_data::Model,
    glam::{Mat4, Quat, Vec3},
};

/// An instance transform.
///
/// It descrides position, rotation and scale.
/// The rotation may be any type that implements [`IntoQuat`].
#[derive(Clone, Copy)]
pub struct Transform {
    /// The position in X, Y, Z coordinates.
    pub pos: Vec3,

    /// The rotation is one of an [`IntoQuat`] type.
    pub rot: Quat,

    /// The scale in X, Y, Z axes.
    pub scl: Vec3,
}

impl Transform {
    pub fn from_position<P>(pos: P) -> Self
    where
        P: Into<Vec3>,
    {
        Self {
            pos: pos.into(),
            ..Default::default()
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scl: Vec3::ONE,
        }
    }
}

impl From<Transform> for Model {
    fn from(Transform { pos, rot, scl }: Transform) -> Self {
        let mat = Mat4::from_scale_rotation_translation(scl, rot, pos);
        Self::from(mat)
    }
}
