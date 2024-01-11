use {
    naga::{AddressSpace, ImageClass, ImageDimension, ScalarKind, Type, TypeInner, VectorSize},
    std::marker::PhantomData,
};

pub trait Scalar {
    const TYPE: ScalarType;
}

impl Scalar for f32 {
    const TYPE: ScalarType = ScalarType::Float;
}

impl Scalar for i32 {
    const TYPE: ScalarType = ScalarType::Sint;
}

impl Scalar for u32 {
    const TYPE: ScalarType = ScalarType::Uint;
}

impl Scalar for bool {
    const TYPE: ScalarType = ScalarType::Bool;
}

#[derive(Clone, Copy)]
pub enum ScalarType {
    Float,
    Sint,
    Uint,
    Bool,
}

impl ScalarType {
    pub(crate) const fn inner(self) -> (ScalarKind, u8) {
        match self {
            Self::Float => (ScalarKind::Float, 4),
            Self::Sint => (ScalarKind::Sint, 4),
            Self::Uint => (ScalarKind::Uint, 4),
            Self::Bool => (ScalarKind::Bool, 1),
        }
    }

    pub(crate) const fn ty(self) -> Type {
        let (kind, width) = self.inner();
        Type {
            name: None,
            inner: TypeInner::Scalar { kind, width },
        }
    }
}

pub struct Vec2<T>(PhantomData<T>);
pub struct Vec3<T>(PhantomData<T>);
pub struct Vec4<T>(PhantomData<T>);

#[derive(Clone, Copy)]
pub enum VectorType {
    Vec2f,
    Vec3f,
    Vec4f,
    Vec2u,
    Vec3u,
    Vec4u,
    Vec2i,
    Vec3i,
    Vec4i,
}

impl VectorType {
    pub(crate) const fn dims(self) -> usize {
        match self {
            Self::Vec2f | Self::Vec2u | Self::Vec2i => 2,
            Self::Vec3f | Self::Vec3u | Self::Vec3i => 3,
            Self::Vec4f | Self::Vec4u | Self::Vec4i => 4,
        }
    }

    pub(crate) const fn ty(self) -> Type {
        match self {
            Self::Vec2f => VEC2F,
            Self::Vec3f => VEC3F,
            Self::Vec4f => VEC4F,
            Self::Vec2u => VEC2U,
            Self::Vec3u => VEC3U,
            Self::Vec4u => VEC4U,
            Self::Vec2i => VEC2I,
            Self::Vec3i => VEC3I,
            Self::Vec4i => VEC4I,
        }
    }
}

pub trait Vector {
    type Scalar;
    const TYPE: VectorType;
}

impl Vector for Vec2<f32> {
    type Scalar = f32;
    const TYPE: VectorType = VectorType::Vec2f;
}

impl Vector for Vec3<f32> {
    type Scalar = f32;
    const TYPE: VectorType = VectorType::Vec3f;
}

impl Vector for Vec4<f32> {
    type Scalar = f32;
    const TYPE: VectorType = VectorType::Vec4f;
}

impl Vector for Vec2<u32> {
    type Scalar = u32;
    const TYPE: VectorType = VectorType::Vec2u;
}

impl Vector for Vec3<u32> {
    type Scalar = u32;
    const TYPE: VectorType = VectorType::Vec3u;
}

impl Vector for Vec4<u32> {
    type Scalar = u32;
    const TYPE: VectorType = VectorType::Vec4u;
}

impl Vector for Vec2<i32> {
    type Scalar = i32;
    const TYPE: VectorType = VectorType::Vec2i;
}

impl Vector for Vec3<i32> {
    type Scalar = i32;
    const TYPE: VectorType = VectorType::Vec3i;
}

impl Vector for Vec4<i32> {
    type Scalar = i32;
    const TYPE: VectorType = VectorType::Vec4i;
}

const VEC2F: Type = vec(VectorSize::Bi, ScalarKind::Float);
const VEC3F: Type = vec(VectorSize::Tri, ScalarKind::Float);
const VEC4F: Type = vec(VectorSize::Quad, ScalarKind::Float);
const VEC2U: Type = vec(VectorSize::Bi, ScalarKind::Uint);
const VEC3U: Type = vec(VectorSize::Tri, ScalarKind::Uint);
const VEC4U: Type = vec(VectorSize::Quad, ScalarKind::Uint);
const VEC2I: Type = vec(VectorSize::Bi, ScalarKind::Sint);
const VEC3I: Type = vec(VectorSize::Tri, ScalarKind::Sint);
const VEC4I: Type = vec(VectorSize::Quad, ScalarKind::Sint);

const fn vec(size: VectorSize, kind: ScalarKind) -> Type {
    Type {
        name: None,
        inner: TypeInner::Vector {
            size,
            kind,
            width: 4,
        },
    }
}

pub struct Mat2;
pub struct Mat3;
pub struct Mat4;

#[derive(Clone, Copy)]
pub enum MatrixType {
    Mat2,
    Mat3,
    Mat4,
}

impl MatrixType {
    pub(crate) const fn dims(self) -> usize {
        match self {
            Self::Mat2 => 2,
            Self::Mat3 => 3,
            Self::Mat4 => 4,
        }
    }

    pub(crate) const fn ty(self) -> Type {
        match self {
            Self::Mat2 => MAT2F,
            Self::Mat3 => MAT3F,
            Self::Mat4 => MAT4F,
        }
    }
}

pub trait Matrix {
    const TYPE: MatrixType;
}

impl Matrix for Mat2 {
    const TYPE: MatrixType = MatrixType::Mat2;
}

impl Matrix for Mat3 {
    const TYPE: MatrixType = MatrixType::Mat3;
}

impl Matrix for Mat4 {
    const TYPE: MatrixType = MatrixType::Mat4;
}

const MAT2F: Type = mat(VectorSize::Bi);
const MAT3F: Type = mat(VectorSize::Tri);
const MAT4F: Type = mat(VectorSize::Quad);

const fn mat(size: VectorSize) -> Type {
    Type {
        name: None,
        inner: TypeInner::Matrix {
            columns: size,
            rows: size,
            width: 4,
        },
    }
}

pub struct Texture2d<T>(PhantomData<T>);

const TEXTURE2DF: Type = texture(ImageDimension::D2, ScalarKind::Float);

#[allow(dead_code)]
const TEXTURE2DU: Type = texture(ImageDimension::D2, ScalarKind::Uint);

#[allow(dead_code)]
const TEXTURE2DI: Type = texture(ImageDimension::D2, ScalarKind::Sint);

const fn texture(dim: ImageDimension, kind: ScalarKind) -> Type {
    Type {
        name: None,
        inner: TypeInner::Image {
            dim,
            arrayed: false,
            class: ImageClass::Sampled { kind, multi: false },
        },
    }
}

pub struct Sampler;

const SAMPLER: Type = Type {
    name: None,
    inner: TypeInner::Sampler { comparison: false },
};

#[derive(Clone, Copy)]
pub enum MemberType {
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
    Tx2df,
    Sampl,
}

impl MemberType {
    pub const fn is_value(self) -> bool {
        matches!(self, Self::Scalar(_) | Self::Vector(_) | Self::Matrix(_))
    }

    pub(crate) const fn ty(self) -> Type {
        match self {
            Self::Scalar(v) => v.ty(),
            Self::Vector(v) => v.ty(),
            Self::Matrix(v) => v.ty(),
            Self::Tx2df => TEXTURE2DF,
            Self::Sampl => SAMPLER,
        }
    }

    pub(crate) const fn address_space(self) -> AddressSpace {
        match self {
            Self::Scalar(_) | Self::Vector(_) | Self::Matrix(_) => AddressSpace::Uniform,
            Self::Tx2df | Self::Sampl => AddressSpace::Handle,
        }
    }
}
