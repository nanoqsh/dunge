pub(crate) use self::proj::{IntoProjection, Projection};

use {
    crate::{
        shader_data::CameraUniform,
        transform::{IntoQuat, Quat},
    },
    glam::{Mat4, Vec3},
    std::cell::Cell,
};

mod proj {
    use super::{Orthographic, Perspective};

    #[derive(Clone, Copy)]
    pub enum Projection {
        Perspective(Perspective),
        Orthographic(Orthographic),
    }

    impl Default for Projection {
        fn default() -> Self {
            Self::Orthographic(Orthographic::default())
        }
    }

    pub trait IntoProjection {
        fn into_projection(self) -> Projection;
    }
}

pub(crate) struct Camera {
    view: View<Projection>,
    cache: Cell<Option<Cache>>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            view: View::default().into_projection_view(),
            cache: Cell::default(),
        }
    }

    pub fn set_view(&mut self, view: View<Projection>) {
        self.view = view;
        self.cache.set(None);
    }

    pub fn uniform(&self, (width, height): (u32, u32)) -> CameraUniform {
        match self.cache.get() {
            Some(Cache { size: (w, h), .. }) if width != w || height != h => {}
            Some(Cache { uniform, .. }) => return uniform,
            None => {}
        }

        let mat = self.view.build_mat((width as f32, height as f32));
        let uniform = CameraUniform::new(mat.to_cols_array_2d());
        self.cache.set(Some(Cache {
            size: (width, height),
            uniform,
        }));

        uniform
    }
}

#[derive(Clone, Copy)]
struct Cache {
    size: (u32, u32),
    uniform: CameraUniform,
}

/// The camera view.
#[derive(Clone, Copy)]
pub struct View<P = Orthographic> {
    /// Eye 3d point.
    pub eye: [f32; 3],

    /// Look at 3d point.
    pub look: [f32; 3],

    /// Up direction.
    pub up: [f32; 3],

    /// Camera projection.
    /// Can be a [`Perspective`] or [`Orthographic`].
    pub proj: P,
}

impl<P> View<P> {
    pub fn into_projection_view(self) -> View<Projection>
    where
        P: IntoProjection,
    {
        View {
            eye: self.eye,
            look: self.look,
            up: self.up,
            proj: self.proj.into_projection(),
        }
    }

    pub fn rotation_quat(&self) -> Quat {
        let mat = Mat4::look_at_rh(Vec3::from(self.eye), Vec3::from(self.look), Vec3::Y);
        let (_, rot, _) = mat.to_scale_rotation_translation();
        Quat(rot.to_array())
    }
}

impl View<Projection> {
    fn build_mat(&self, (width, height): (f32, f32)) -> Mat4 {
        let proj = match self.proj {
            Projection::Perspective(Perspective { fovy, znear, zfar }) => {
                Mat4::perspective_rh(fovy, width / height, znear, zfar)
            }
            Projection::Orthographic(Orthographic {
                width_factor,
                height_factor,
                near,
                far,
            }) => {
                let wh = width * width_factor * 0.5;
                let left = -wh;
                let right = wh;

                let hh = height * height_factor * 0.5;
                let bottom = -hh;
                let top = hh;

                Mat4::orthographic_rh(left, right, bottom, top, near, far)
            }
        };

        let view = Mat4::look_at_rh(self.eye.into(), self.look.into(), self.up.into());
        proj * view
    }
}

impl Default for View<Orthographic> {
    fn default() -> Self {
        Self {
            eye: [0., 0., 1.],
            look: [0.; 3],
            up: [0., 1., 0.],
            proj: Orthographic::default(),
        }
    }
}

impl<P> IntoQuat for View<P> {
    fn into_quat(self) -> Quat {
        self.rotation_quat()
    }
}

/// Perspective projection.
#[derive(Clone, Copy)]
pub struct Perspective {
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            fovy: 1.6,
            znear: 0.1,
            zfar: 100.,
        }
    }
}

impl IntoProjection for Perspective {
    fn into_projection(self) -> Projection {
        Projection::Perspective(self)
    }
}

/// Orthographic projection.
#[derive(Clone, Copy)]
pub struct Orthographic {
    pub width_factor: f32,
    pub height_factor: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Orthographic {
    fn default() -> Self {
        Self {
            width_factor: 1.,
            height_factor: 1.,
            near: -100.,
            far: 100.,
        }
    }
}

impl IntoProjection for Orthographic {
    fn into_projection(self) -> Projection {
        Projection::Orthographic(self)
    }
}
