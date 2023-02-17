#![allow(clippy::wildcard_imports)]

pub(crate) use self::proj::{IntoProjection, Projection};

use {
    crate::{
        layout::Plain,
        shader,
        transform::{IntoQuat, Quat},
    },
    std::cell::Cell,
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
    size: Cell<Option<(u32, u32)>>,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Camera {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            *,
        };

        const BINDING: u32 = {
            assert!(shader::TEXTURED_CAMERA_BINDING == shader::COLOR_CAMERA_BINDING);
            shader::TEXTURED_CAMERA_BINDING
        };

        let uniform = CameraUniform {
            view_proj: IDENTITY,
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: uniform.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[BindGroupEntry {
                binding: BINDING,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera bind group"),
        });

        Self {
            view: View::<Orthographic>::default().into_projection_view(),
            size: Cell::new(None),
            buffer,
            bind_group,
        }
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.view = view;
        self.size.set(None);
    }

    pub(crate) fn resize(&self, size @ (width, height): (u32, u32), queue: &Queue) {
        if self
            .size
            .get()
            .map(|(w, h)| width == w && height == h)
            .unwrap_or_default()
        {
            return;
        }

        self.size.set(Some(size));
        let uniform = CameraUniform {
            view_proj: self.view.build_mat((width as f32, height as f32)),
        };
        queue.write_buffer(&self.buffer, 0, uniform.as_bytes());
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

/// The camera view.
#[derive(Clone, Copy)]
pub struct View<P = Perspective> {
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

    pub(crate) fn rotation_quat(&self) -> Quat {
        let [xe, ye, ze] = self.eye;
        let [xl, yl, zl] = self.look;
        let [sx, sy, sz] = normalize([xe - xl, ye - yl, ze - zl]);

        let pitch = sy.asin();
        let angle = sx.atan2(sz);
        let (pitch_sin, pitch_cos) = (pitch * 0.5).sin_cos();
        let (angle_sin, angle_cos) = (-angle * 0.5).sin_cos();

        Quat([
            pitch_sin * angle_cos,
            pitch_cos * angle_sin,
            pitch_sin * angle_sin,
            pitch_cos * angle_cos,
        ])
    }
}

impl View<Projection> {
    fn build_mat(&self, (width, height): (f32, f32)) -> [[f32; 4]; 4] {
        let proj = match self.proj {
            Projection::Perspective(Perspective { fovy, znear, zfar }) => {
                let (sin_fov, cos_fov) = (0.5 * fovy).sin_cos();
                let h = cos_fov / sin_fov;
                let w = (h * height) / width;
                let r = zfar / (znear - zfar);

                [
                    [w, 0., 0., 0.],
                    [0., h, 0., 0.],
                    [0., 0., r, -1.],
                    [0., 0., r * znear, 0.],
                ]
            }
            Projection::Orthographic(Orthographic {
                width_factor,
                height_factor,
                near,
                far,
            }) => {
                let factor_width = 1. / (width * width_factor);
                let factor_height = 1. / (height * height_factor);
                let r = 1. / (near - far);

                [
                    [factor_width + factor_width, 0., 0., 0.],
                    [0., factor_height + factor_height, 0., 0.],
                    [0., 0., r, 0.],
                    [factor_width, factor_height, r * near, 1.],
                ]
            }
        };

        let view = {
            let [xe, ye, ze] = self.eye;
            let [xl, yl, zl] = self.look;
            let [xf, yf, zf] = normalize([xe - xl, ye - yl, ze - zl]);
            let [xr, yr, zr] = normalize(cross([0., 1., 0.], [xf, yf, zf]));
            let [xu, yu, zu] = cross([xf, yf, zf], [xr, yr, zr]);

            let tx = xr * xe + yr * ye + zr * ze;
            let ty = xu * xe + yu * ye + zu * ze;
            let tz = xf * xe + yf * ye + zf * ze;

            [
                [xr, xu, xf, 0.],
                [yr, yu, yf, 0.],
                [zr, zu, zf, 0.],
                [-tx, -ty, -tz, 1.],
            ]
        };

        mul_mat4(proj, view)
    }
}

impl<P> Default for View<P>
where
    P: Default,
{
    fn default() -> Self {
        Self {
            eye: [0., 0., 1.],
            look: [0.; 3],
            proj: P::default(),
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

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

unsafe impl Plain for CameraUniform {}

const IDENTITY: [[f32; 4]; 4] = [
    [1., 0., 0., 0.],
    [0., 1., 0., 0.],
    [0., 0., 1., 0.],
    [0., 0., 0., 1.],
];

fn normalize([x, y, z]: [f32; 3]) -> [f32; 3] {
    let len = (x * x + y * y + z * z).sqrt();
    if len == 0. {
        [0., 1., 0.]
    } else {
        [x / len, y / len, z / len]
    }
}

fn cross([xa, ya, za]: [f32; 3], [xb, yb, zb]: [f32; 3]) -> [f32; 3] {
    [ya * zb - yb * za, za * xb - zb * xa, xa * yb - xb * ya]
}

fn mul_vector(mat: [[f32; 4]; 4], [x, y, z, w]: [f32; 4]) -> [f32; 4] {
    let [[xa, ya, za, wa], [xb, yb, zb, wb], [xc, yc, zc, wc], [xd, yd, zd, wd]] = mat;

    [
        x * xa + y * xb + z * xc + w * xd,
        x * ya + y * yb + z * yc + w * yd,
        x * za + y * zb + z * zc + w * zd,
        x * wa + y * wb + z * wc + w * wd,
    ]
}

fn mul_mat4(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    b.map(|v| mul_vector(a, v))
}
