use {
    crate::shader_data::ModelTransform,
    glam::{Mat4, Quat, Vec3},
    std::cell::Cell,
};

#[derive(Default)]
pub(crate) struct Camera {
    view: View,
    cache: Cell<Option<Cache>>,
}

impl Camera {
    pub fn update_view(&mut self, view: View) {
        self.view = view;
        self.cache.set(None);
    }

    pub fn model(&self, (width, height): (u32, u32)) -> ModelTransform {
        match self.cache.get() {
            Some(Cache { size: (w, h), .. }) if width != w || height != h => {}
            Some(Cache { model, .. }) => return model,
            None => {}
        }

        let model = self.view.model((width as f32, height as f32));
        self.cache.set(Some(Cache {
            size: (width, height),
            model,
        }));

        model
    }
}

#[derive(Clone, Copy)]
struct Cache {
    size: (u32, u32),
    model: ModelTransform,
}

/// The camera view.
#[derive(Clone, Copy)]
pub struct View {
    /// Eye 3d point.
    pub eye: Vec3,

    /// Look at 3d point.
    pub look: Vec3,

    /// Up direction.
    pub up: Vec3,

    /// Camera projection.
    pub proj: Projection,
}

impl View {
    pub fn rotation(&self) -> Quat {
        let mat = Mat4::look_at_rh(self.eye, self.look, self.up);
        let (_, rot, _) = mat.to_scale_rotation_translation();
        rot
    }
}

impl View {
    fn model(&self, (width, height): (f32, f32)) -> ModelTransform {
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

        let view = Mat4::look_at_rh(self.eye, self.look, self.up);
        ModelTransform::from(proj * view)
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            eye: Vec3::Z,
            look: Vec3::ZERO,
            up: Vec3::Y,
            proj: Projection::default(),
        }
    }
}

/// The camera projection.
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

impl From<Perspective> for Projection {
    fn from(v: Perspective) -> Self {
        Self::Perspective(v)
    }
}

impl From<Orthographic> for Projection {
    fn from(v: Orthographic) -> Self {
        Self::Orthographic(v)
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
