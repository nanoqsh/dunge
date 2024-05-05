//! Shader types.

use {
    naga::{AddressSpace, ImageClass, ImageDimension, ScalarKind, Type, TypeInner, VectorSize},
    std::marker::PhantomData,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
}

impl ValueType {
    pub(crate) const fn ty(self) -> Type {
        match self {
            Self::Scalar(v) => v.ty(),
            Self::Vector(v) => v.ty(),
            Self::Matrix(v) => v.ty(),
        }
    }

    const fn into_scalar(self) -> ScalarType {
        match self {
            Self::Scalar(v) => v,
            _ => panic!("non-scalar type"),
        }
    }

    const fn into_vector(self) -> VectorType {
        match self {
            Self::Vector(v) => v,
            _ => panic!("non-vector type"),
        }
    }

    const fn into_matrix(self) -> MatrixType {
        match self {
            Self::Matrix(v) => v,
            _ => panic!("non-matrix type"),
        }
    }
}

/// The trait for types used inside a shader.
pub trait Value {
    const VALUE_TYPE: ValueType;
}

impl Value for f32 {
    const VALUE_TYPE: ValueType = ValueType::Scalar(ScalarType::Float);
}

impl Value for i32 {
    const VALUE_TYPE: ValueType = ValueType::Scalar(ScalarType::Sint);
}

impl Value for u32 {
    const VALUE_TYPE: ValueType = ValueType::Scalar(ScalarType::Uint);
}

impl Value for bool {
    const VALUE_TYPE: ValueType = ValueType::Scalar(ScalarType::Bool);
}

/// The trait for types used inside a shader as scalars.
pub trait Scalar: Value {
    const TYPE: ScalarType = Self::VALUE_TYPE.into_scalar();
}

impl Scalar for f32 {}
impl Scalar for i32 {}
impl Scalar for u32 {}
impl Scalar for bool {}

/// The trait for types used inside a shader as numbers.
pub trait Number: Scalar {}

impl Number for f32 {}
impl Number for i32 {}
impl Number for u32 {}

#[derive(Clone, Copy, PartialEq, Eq)]
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
        use naga::Scalar;

        let (kind, width) = self.inner();
        Type {
            name: None,
            inner: TypeInner::Scalar(Scalar { kind, width }),
        }
    }
}

pub struct Vec2<T>(PhantomData<T>);
pub struct Vec3<T>(PhantomData<T>);
pub struct Vec4<T>(PhantomData<T>);

#[derive(Clone, Copy, PartialEq, Eq)]
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

/// The trait for types used inside a shader as vectors.
pub trait Vector: Value {
    type Scalar;
    const TYPE: VectorType = Self::VALUE_TYPE.into_vector();
}

impl Value for Vec2<f32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec2f);
}

impl Value for Vec3<f32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec3f);
}

impl Value for Vec4<f32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec4f);
}

impl Value for Vec2<u32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec2u);
}

impl Value for Vec3<u32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec3u);
}

impl Value for Vec4<u32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec4u);
}

impl Value for Vec2<i32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec2i);
}

impl Value for Vec3<i32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec3i);
}

impl Value for Vec4<i32> {
    const VALUE_TYPE: ValueType = ValueType::Vector(VectorType::Vec4i);
}

impl Vector for Vec2<f32> {
    type Scalar = f32;
}

impl Vector for Vec3<f32> {
    type Scalar = f32;
}

impl Vector for Vec4<f32> {
    type Scalar = f32;
}

impl Vector for Vec2<u32> {
    type Scalar = u32;
}

impl Vector for Vec3<u32> {
    type Scalar = u32;
}

impl Vector for Vec4<u32> {
    type Scalar = u32;
}

impl Vector for Vec2<i32> {
    type Scalar = i32;
}

impl Vector for Vec3<i32> {
    type Scalar = i32;
}

impl Vector for Vec4<i32> {
    type Scalar = i32;
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
    use naga::Scalar;

    Type {
        name: None,
        inner: TypeInner::Vector {
            size,
            scalar: Scalar { kind, width: 4 },
        },
    }
}

pub struct Mat2;
pub struct Mat3;
pub struct Mat4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MatrixType {
    Mat2,
    Mat3,
    Mat4,
}

impl MatrixType {
    pub const fn dims(self) -> u32 {
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

    pub const fn vector_type(self) -> VectorType {
        match self {
            Self::Mat2 => VectorType::Vec2f,
            Self::Mat3 => VectorType::Vec3f,
            Self::Mat4 => VectorType::Vec4f,
        }
    }
}

/// The trait for types used inside a shader as matrices.
pub trait Matrix: Value {
    const TYPE: MatrixType = Self::VALUE_TYPE.into_matrix();
}

impl Value for Mat2 {
    const VALUE_TYPE: ValueType = ValueType::Matrix(MatrixType::Mat2);
}

impl Value for Mat3 {
    const VALUE_TYPE: ValueType = ValueType::Matrix(MatrixType::Mat3);
}

impl Value for Mat4 {
    const VALUE_TYPE: ValueType = ValueType::Matrix(MatrixType::Mat4);
}

impl Matrix for Mat2 {}
impl Matrix for Mat3 {}
impl Matrix for Mat4 {}

const MAT2F: Type = mat(VectorSize::Bi);
const MAT3F: Type = mat(VectorSize::Tri);
const MAT4F: Type = mat(VectorSize::Quad);

const fn mat(size: VectorSize) -> Type {
    use naga::Scalar;

    Type {
        name: None,
        inner: TypeInner::Matrix {
            columns: size,
            rows: size,
            scalar: Scalar {
                width: 4,
                kind: ScalarKind::Float,
            },
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemberType {
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
    Tx2df,
    Sampl,
}

impl MemberType {
    pub const fn from_value(v: ValueType) -> Self {
        match v {
            ValueType::Scalar(v) => Self::Scalar(v),
            ValueType::Vector(v) => Self::Vector(v),
            ValueType::Matrix(v) => Self::Matrix(v),
        }
    }

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
