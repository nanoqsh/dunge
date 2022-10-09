use {
    crate::Size,
    glam::{Mat4, Vec3},
    wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue},
};

pub(crate) struct Camera {
    view: View<Projection>,
    uniform: CameraUniform,
    buffer: Buffer,
    bind_group: BindGroup,
}

impl Camera {
    pub(crate) fn new(device: &Device, layout: &BindGroupLayout) -> Self {
        use {
            std::{mem, slice},
            wgpu::{
                util::{BufferInitDescriptor, DeviceExt},
                *,
            },
        };

        let uniform = CameraUniform {
            mat: *Mat4::IDENTITY.as_ref(),
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: unsafe {
                slice::from_raw_parts(uniform.as_ptr().cast(), mem::size_of::<CameraUniform>())
            },
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
            view: View::default().into_projection(),
            uniform,
            buffer,
            bind_group,
        }
    }

    pub(crate) fn set_view(&mut self, view: View<Projection>) {
        self.view = view;
    }

    pub(crate) fn resize(&mut self, (width, height): Size, queue: &Queue) {
        use std::{mem, slice};

        let aspect = width.get() as f32 / height.get() as f32;
        self.uniform.mat = *self.view.build_mat(aspect).as_ref();

        let data = unsafe {
            slice::from_raw_parts(
                self.uniform.as_ptr().cast(),
                mem::size_of::<CameraUniform>(),
            )
        };
        queue.write_buffer(&self.buffer, 0, data);
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

#[derive(Clone, Copy)]
pub struct View<P> {
    pub eye: [f32; 3],
    pub look: [f32; 3],
    pub proj: P,
}

impl<P> View<P> {
    pub(crate) fn into_projection(self) -> View<Projection>
    where
        P: Into<Projection>,
    {
        View {
            eye: self.eye,
            look: self.look,
            proj: self.proj.into(),
        }
    }
}

impl View<Projection> {
    pub(crate) fn build_mat(&self, aspect: f32) -> Mat4 {
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

#[derive(Clone, Copy)]
pub enum Projection {
    Perspective(Perspective),
    Orthographic(Orthographic),
}

#[derive(Clone, Copy)]
pub struct Perspective {
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl From<Perspective> for Projection {
    fn from(v: Perspective) -> Self {
        Self::Perspective(v)
    }
}

#[derive(Clone, Copy)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl From<Orthographic> for Projection {
    fn from(v: Orthographic) -> Self {
        Self::Orthographic(v)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CameraUniform {
    mat: [f32; 16],
}

impl CameraUniform {
    fn as_ptr(&self) -> *const Self {
        self as *const Self
    }
}
