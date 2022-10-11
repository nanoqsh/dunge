pub(crate) use self::proj::{IntoProjection, Projection};

use {
    crate::{instance::Rotation, layout::Plain},
    glam::{Mat4, Vec3},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

mod proj {
    use super::{Orthographic, Perspective};

    #[derive(Clone, Copy)]
    pub enum Projection {
        Perspective(Perspective),
        Orthographic(Orthographic),
    }

    pub trait IntoProjection {
        fn into_projection(self) -> Projection;
    }
}

pub(crate) struct Camera {
    view: View<Projection>,
    uniform: CameraUniform,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Camera {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            *,
        };

        let uniform = CameraUniform {
            view_proj: *Mat4::IDENTITY.as_ref(),
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: uniform.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera bind group"),
        });

        Self {
            view: View::default().into_projection_view(),
            uniform,
            buffer,
            bind_group,
        }
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.view = view;
    }

    pub(crate) fn resize(&mut self, (width, height): (f32, f32), queue: &Queue) {
        let aspect = width / height;
        let view_proj = self.view.build_mat(aspect);
        self.uniform.view_proj = *view_proj.as_ref();

        queue.write_buffer(&self.buffer, 0, self.uniform.as_bytes());
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

/// The camera view.
#[derive(Clone, Copy)]
pub struct View<P> {
    /// Eye 3d point.
    pub eye: [f32; 3],

    /// Look at 3d point.
    pub look: [f32; 3],

    /// Camera projection.
    /// Can be a [`Perspective`] or [`Orthographic`].
    pub proj: P,
}

impl<P> View<P> {
    pub(crate) fn into_projection_view(self) -> View<Projection>
    where
        P: IntoProjection,
    {
        View {
            eye: self.eye,
            look: self.look,
            proj: self.proj.into_projection(),
        }
    }

    pub(crate) fn rotation_quat(&self) -> [f32; 4] {
        let [xe, ye, ze] = self.eye;
        let [xl, yl, zl] = self.look;
        let [sx, sy, sz] = [xe - xl, ye - yl, ze - zl];
        let len = (sx * sx + sy * sy + sz * sz).sqrt();

        let [sx, sy, sz] = if len == 0. {
            [0., 1., 0.]
        } else {
            [sx / len, sy / len, sz / len]
        };

        let pitch = sy.asin();
        let angle = sx.atan2(sz);
        let (asin, acos) = (pitch * 0.5).sin_cos();
        let (bsin, bcos) = (-angle * 0.5).sin_cos();

        [asin * bcos, acos * bsin, asin * bsin, acos * bcos]
    }
}

impl View<Projection> {
    fn build_mat(&self, aspect: f32) -> Mat4 {
        let proj = match self.proj {
            Projection::Perspective(Perspective { fovy, znear, zfar }) => {
                Mat4::perspective_rh(fovy, aspect, znear, zfar)
            }
            Projection::Orthographic(Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            }) => Mat4::orthographic_rh(left, right, bottom, top, near, far),
        };

        let view = Mat4::look_at_rh(self.eye.into(), self.look.into(), Vec3::Y);

        proj * view
    }
}

impl Default for View<Perspective> {
    fn default() -> Self {
        Self {
            eye: [0.; 3],
            look: [0.; 3],
            proj: Perspective {
                fovy: 1.58,
                znear: 0.1,
                zfar: 100.,
            },
        }
    }
}

impl<P> Rotation for View<P> {
    fn into_quat(self) -> [f32; 4] {
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

impl IntoProjection for Perspective {
    fn into_projection(self) -> Projection {
        Projection::Perspective(self)
    }
}

/// Orthographic projection.
#[derive(Clone, Copy)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl IntoProjection for Orthographic {
    fn into_projection(self) -> Projection {
        Projection::Orthographic(self)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CameraUniform {
    view_proj: [f32; 16],
}

unsafe impl Plain for CameraUniform {}
