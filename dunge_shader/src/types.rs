//! Shader types.

use std::{marker::PhantomData, num::NonZeroU32};

pub(crate) trait AddType {
    fn add_type(&mut self, ty: naga::Type) -> naga::Handle<naga::Type>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
    Array(ArrayType),
}

impl ValueType {
    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        match self {
            Self::Scalar(v) => v.ty(add),
            Self::Vector(v) => v.ty(add),
            Self::Matrix(v) => v.ty(add),
            Self::Array(v) => v.ty(add),
        }
    }

    fn stride(self) -> u32 {
        match self {
            Self::Scalar(v) => v.stride(),
            Self::Vector(v) => v.stride(),
            Self::Matrix(v) => v.stride(),
            Self::Array(v) => v.stride(),
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
    pub(crate) fn inner(self) -> (naga::ScalarKind, u8) {
        match self {
            Self::Float => (naga::ScalarKind::Float, 4),
            Self::Sint => (naga::ScalarKind::Sint, 4),
            Self::Uint => (naga::ScalarKind::Uint, 4),
            Self::Bool => (naga::ScalarKind::Bool, 1),
        }
    }

    pub(crate) fn naga(self) -> naga::Type {
        let (kind, width) = self.inner();

        naga::Type {
            name: None,
            inner: naga::TypeInner::Scalar(naga::Scalar { kind, width }),
        }
    }

    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        add.add_type(self.naga())
    }

    fn stride(self) -> u32 {
        match self {
            Self::Float | Self::Sint | Self::Uint => 4,
            Self::Bool => 1,
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
    pub(crate) const fn dims(self) -> u32 {
        match self {
            Self::Vec2f | Self::Vec2u | Self::Vec2i => 2,
            Self::Vec3f | Self::Vec3u | Self::Vec3i => 3,
            Self::Vec4f | Self::Vec4u | Self::Vec4i => 4,
        }
    }

    pub(crate) fn naga(self) -> naga::Type {
        const fn new(size: naga::VectorSize, kind: naga::ScalarKind) -> naga::Type {
            naga::Type {
                name: None,
                inner: naga::TypeInner::Vector {
                    size,
                    scalar: naga::Scalar { kind, width: 4 },
                },
            }
        }

        match self {
            Self::Vec2f => const { new(naga::VectorSize::Bi, naga::ScalarKind::Float) },
            Self::Vec3f => const { new(naga::VectorSize::Tri, naga::ScalarKind::Float) },
            Self::Vec4f => const { new(naga::VectorSize::Quad, naga::ScalarKind::Float) },
            Self::Vec2u => const { new(naga::VectorSize::Bi, naga::ScalarKind::Uint) },
            Self::Vec3u => const { new(naga::VectorSize::Tri, naga::ScalarKind::Uint) },
            Self::Vec4u => const { new(naga::VectorSize::Quad, naga::ScalarKind::Uint) },
            Self::Vec2i => const { new(naga::VectorSize::Bi, naga::ScalarKind::Sint) },
            Self::Vec3i => const { new(naga::VectorSize::Tri, naga::ScalarKind::Sint) },
            Self::Vec4i => const { new(naga::VectorSize::Quad, naga::ScalarKind::Sint) },
        }
    }

    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        add.add_type(self.naga())
    }

    fn stride(self) -> u32 {
        self.dims() * 4
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

    pub(crate) fn naga(self) -> naga::Type {
        const fn new(size: naga::VectorSize) -> naga::Type {
            naga::Type {
                name: None,
                inner: naga::TypeInner::Matrix {
                    columns: size,
                    rows: size,
                    scalar: naga::Scalar {
                        width: 4,
                        kind: naga::ScalarKind::Float,
                    },
                },
            }
        }

        match self {
            Self::Mat2 => const { new(naga::VectorSize::Bi) },
            Self::Mat3 => const { new(naga::VectorSize::Tri) },
            Self::Mat4 => const { new(naga::VectorSize::Quad) },
        }
    }

    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        add.add_type(self.naga())
    }

    fn stride(self) -> u32 {
        let dims = self.dims();
        dims * dims * 4
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

pub struct Array<V, const N: usize>(PhantomData<V>);

impl<V, const N: usize> Value for Array<V, N>
where
    V: Value,
{
    const VALUE_TYPE: ValueType = ValueType::Array(ArrayType {
        base: &V::VALUE_TYPE,
        size: NonZeroU32::new(N as u32).expect("array size cannot be zero"),
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ArrayType {
    pub base: &'static ValueType,
    pub size: NonZeroU32,
}

impl ArrayType {
    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        let base = self.base.ty(add);
        let ty = naga::Type {
            name: None,
            inner: naga::TypeInner::Array {
                base,
                size: naga::ArraySize::Constant(self.size),
                stride: self.base.stride(),
            },
        };

        add.add_type(ty)
    }

    fn stride(self) -> u32 {
        self.base.stride() * self.size.get()
    }
}

pub trait Member {
    const MEMBER_TYPE: MemberType;
}

impl<V> Member for V
where
    V: Value,
{
    const MEMBER_TYPE: MemberType = MemberType::from_value(V::VALUE_TYPE);
}

pub struct DynamicArray<V>(PhantomData<V>);

impl<V> Member for DynamicArray<V>
where
    V: Value,
{
    const MEMBER_TYPE: MemberType = MemberType::DynamicArrayType(DynamicArrayType {
        base: &V::VALUE_TYPE,
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DynamicArrayType {
    pub base: &'static ValueType,
}

impl DynamicArrayType {
    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        let base = self.base.ty(add);
        let ty = naga::Type {
            name: None,
            inner: naga::TypeInner::Array {
                base,
                size: naga::ArraySize::Dynamic,
                stride: self.base.stride(),
            },
        };

        add.add_type(ty)
    }
}

pub struct Texture2d<T>(PhantomData<T>);

impl Member for Texture2d<f32> {
    const MEMBER_TYPE: MemberType = MemberType::Tx2df;
}

pub struct Sampler;

impl Member for Sampler {
    const MEMBER_TYPE: MemberType = MemberType::Sampl;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemberType {
    Scalar(ScalarType),
    Vector(VectorType),
    Matrix(MatrixType),
    Array(ArrayType),
    DynamicArrayType(DynamicArrayType),
    Tx2df,
    Sampl,
}

impl MemberType {
    pub const fn from_value(v: ValueType) -> Self {
        match v {
            ValueType::Scalar(v) => Self::Scalar(v),
            ValueType::Vector(v) => Self::Vector(v),
            ValueType::Matrix(v) => Self::Matrix(v),
            ValueType::Array(v) => Self::Array(v),
        }
    }

    pub(crate) fn ty<A>(self, add: &mut A) -> naga::Handle<naga::Type>
    where
        A: AddType,
    {
        const fn texture(dim: naga::ImageDimension, kind: naga::ScalarKind) -> naga::Type {
            naga::Type {
                name: None,
                inner: naga::TypeInner::Image {
                    dim,
                    arrayed: false,
                    class: naga::ImageClass::Sampled { kind, multi: false },
                },
            }
        }

        match self {
            Self::Scalar(v) => v.ty(add),
            Self::Vector(v) => v.ty(add),
            Self::Matrix(v) => v.ty(add),
            Self::Array(v) => v.ty(add),
            Self::DynamicArrayType(v) => v.ty(add),
            Self::Tx2df => {
                add.add_type(const { texture(naga::ImageDimension::D2, naga::ScalarKind::Float) })
            }
            Self::Sampl => add.add_type(
                const {
                    naga::Type {
                        name: None,
                        inner: naga::TypeInner::Sampler { comparison: false },
                    }
                },
            ),
        }
    }

    pub(crate) fn address_space(self, mutable: bool) -> naga::AddressSpace {
        match self {
            Self::Scalar(_) | Self::Vector(_) | Self::Matrix(_) => naga::AddressSpace::Uniform,
            Self::Array(_) | Self::DynamicArrayType(_) => {
                let mut access = naga::StorageAccess::LOAD;
                access.set(naga::StorageAccess::STORE, mutable);
                naga::AddressSpace::Storage { access }
            }
            Self::Tx2df | Self::Sampl => naga::AddressSpace::Handle,
        }
    }
}

/// Some values require an indirect load to be read from a global variable.
pub const fn indirect_load<M>() -> bool
where
    M: Member,
{
    matches!(
        M::MEMBER_TYPE,
        MemberType::Scalar(_) | MemberType::Vector(_) | MemberType::Matrix(_),
    )
}

pub enum Immutable {}
pub enum Mutable {}

pub trait Mutability {
    const MUTABLE: bool;
}

impl Mutability for Immutable {
    const MUTABLE: bool = false;
}

impl Mutability for Mutable {
    const MUTABLE: bool = true;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemberData {
    pub ty: MemberType,
    pub mutable: bool,
}
