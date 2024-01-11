use {
    crate::{
        eval::{Eval, Expr, GetEntry, Op},
        types,
    },
    std::{marker::PhantomData, ops},
};

pub struct Ret<A, T> {
    a: A,
    t: PhantomData<T>,
}

impl<A, T> Ret<A, T> {
    pub(crate) const fn new(a: A) -> Self {
        Self { a, t: PhantomData }
    }

    pub(crate) fn get(self) -> A {
        self.a
    }
}

impl<A, T> Clone for Ret<A, T>
where
    A: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, T> Copy for Ret<A, T> where A: Copy {}

pub struct Binary<A, B> {
    a: A,
    b: B,
    op: Op,
}

impl<A, B, O, E> Eval<E> for Ret<Binary<A, B>, O>
where
    A: Eval<E>,
    B: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Binary { a, b, op } = self.get();
        let x = a.eval(en);
        let y = b.eval(en);
        en.get_entry().binary(op, x, y)
    }
}

macro_rules! impl_op {
    ($o:ident :: $f:ident ( $a:ty, $b:ty ) -> $r:ty) => {
        impl<A> ops::$o<Ret<A, $b>> for $a {
            type Output = Ret<Binary<$a, Ret<A, $b>>, $r>;

            fn $f(self, b: Ret<A, $b>) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Op::$o,
                })
            }
        }

        impl<A> ops::$o<$a> for Ret<A, $b> {
            type Output = Ret<Binary<Ret<A, $b>, $a>, $r>;

            fn $f(self, b: $a) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Op::$o,
                })
            }
        }

        impl<A, B> ops::$o<Ret<B, $b>> for Ret<A, $a> {
            type Output = Ret<Binary<Ret<A, $a>, Ret<B, $b>>, $r>;

            fn $f(self, b: Ret<B, $b>) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Op::$o,
                })
            }
        }
    };
}

impl_op!(Add::add(f32, f32) -> f32);
impl_op!(Sub::sub(f32, f32) -> f32);
impl_op!(Mul::mul(f32, f32) -> f32);
impl_op!(Div::div(f32, f32) -> f32);
impl_op!(Rem::rem(f32, f32) -> f32);
impl_op!(Add::add(i32, i32) -> i32);
impl_op!(Sub::sub(i32, i32) -> i32);
impl_op!(Mul::mul(i32, i32) -> i32);
impl_op!(Div::div(i32, i32) -> i32);
impl_op!(Rem::rem(i32, i32) -> i32);
impl_op!(Add::add(u32, u32) -> u32);
impl_op!(Sub::sub(u32, u32) -> u32);
impl_op!(Mul::mul(u32, u32) -> u32);
impl_op!(Div::div(u32, u32) -> u32);
impl_op!(Rem::rem(u32, u32) -> u32);

impl_op!(Add::add(types::Vec2<f32>, types::Vec2<f32>) -> types::Vec2<f32>);
impl_op!(Add::add(types::Vec3<f32>, types::Vec3<f32>) -> types::Vec3<f32>);
impl_op!(Add::add(types::Vec4<f32>, types::Vec4<f32>) -> types::Vec4<f32>);
impl_op!(Sub::sub(types::Vec2<f32>, types::Vec2<f32>) -> types::Vec2<f32>);
impl_op!(Sub::sub(types::Vec3<f32>, types::Vec3<f32>) -> types::Vec3<f32>);
impl_op!(Sub::sub(types::Vec4<f32>, types::Vec4<f32>) -> types::Vec4<f32>);
impl_op!(Mul::mul(f32, types::Vec2<f32>) -> types::Vec2<f32>);
impl_op!(Mul::mul(types::Vec2<f32>, f32) -> types::Vec2<f32>);
impl_op!(Mul::mul(f32, types::Vec3<f32>) -> types::Vec3<f32>);
impl_op!(Mul::mul(types::Vec3<f32>, f32) -> types::Vec3<f32>);
impl_op!(Mul::mul(f32, types::Vec4<f32>) -> types::Vec4<f32>);
impl_op!(Mul::mul(types::Vec4<f32>, f32) -> types::Vec4<f32>);

impl_op!(Add::add(types::Mat2, types::Mat2) -> types::Mat2);
impl_op!(Add::add(types::Mat3, types::Mat3) -> types::Mat3);
impl_op!(Add::add(types::Mat4, types::Mat4) -> types::Mat4);
impl_op!(Sub::sub(types::Mat2, types::Mat2) -> types::Mat2);
impl_op!(Sub::sub(types::Mat3, types::Mat3) -> types::Mat3);
impl_op!(Sub::sub(types::Mat4, types::Mat4) -> types::Mat4);
impl_op!(Mul::mul(types::Mat2, types::Mat2) -> types::Mat2);
impl_op!(Mul::mul(types::Mat3, types::Mat3) -> types::Mat3);
impl_op!(Mul::mul(types::Mat4, types::Mat4) -> types::Mat4);
impl_op!(Mul::mul(f32, types::Mat2) -> types::Mat2);
impl_op!(Mul::mul(f32, types::Mat3) -> types::Mat3);
impl_op!(Mul::mul(f32, types::Mat4) -> types::Mat4);
impl_op!(Mul::mul(types::Mat2, types::Vec2<f32>) -> types::Vec2<f32>);
impl_op!(Mul::mul(types::Mat3, types::Vec3<f32>) -> types::Vec3<f32>);
impl_op!(Mul::mul(types::Mat4, types::Vec4<f32>) -> types::Vec4<f32>);
