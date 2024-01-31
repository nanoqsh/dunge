use {
    crate::eval::{Eval, Expr, GetEntry, Op, Vs},
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

type Out<T> = <T as Eval<Vs>>::Out;

macro_rules! impl_op {
    ($o:ident :: $f:ident ( $a:ty, $b:ty ) -> $r:ty) => {
        impl<A> ops::$o<Ret<A, Out<$b>>> for $a {
            type Output = Ret<Binary<Self, Ret<A, Out<$b>>>, Out<$r>>;

            fn $f(self, b: Ret<A, Out<$b>>) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Op::$o,
                })
            }
        }

        impl<A> ops::$o<$a> for Ret<A, Out<$b>> {
            type Output = Ret<Binary<Self, $a>, Out<$r>>;

            fn $f(self, b: $a) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Op::$o,
                })
            }
        }

        impl<A, B> ops::$o<Ret<B, Out<$b>>> for Ret<A, Out<$a>> {
            type Output = Ret<Binary<Self, Ret<B, Out<$b>>>, Out<$r>>;

            fn $f(self, b: Ret<B, Out<$b>>) -> Self::Output {
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

impl_op!(Add::add(glam::Vec2, glam::Vec2) -> glam::Vec2);
impl_op!(Add::add(glam::Vec3, glam::Vec3) -> glam::Vec3);
impl_op!(Add::add(glam::Vec4, glam::Vec4) -> glam::Vec4);
impl_op!(Sub::sub(glam::Vec2, glam::Vec2) -> glam::Vec2);
impl_op!(Sub::sub(glam::Vec3, glam::Vec3) -> glam::Vec3);
impl_op!(Sub::sub(glam::Vec4, glam::Vec4) -> glam::Vec4);
impl_op!(Mul::mul(f32, glam::Vec2) -> glam::Vec2);
impl_op!(Mul::mul(glam::Vec2, f32) -> glam::Vec2);
impl_op!(Mul::mul(f32, glam::Vec3) -> glam::Vec3);
impl_op!(Mul::mul(glam::Vec3, f32) -> glam::Vec3);
impl_op!(Mul::mul(f32, glam::Vec4) -> glam::Vec4);
impl_op!(Mul::mul(glam::Vec4, f32) -> glam::Vec4);

impl_op!(Add::add(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_op!(Add::add(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_op!(Add::add(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_op!(Sub::sub(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_op!(Sub::sub(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_op!(Sub::sub(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_op!(Mul::mul(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_op!(Mul::mul(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_op!(Mul::mul(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_op!(Mul::mul(f32, glam::Mat2) -> glam::Mat2);
impl_op!(Mul::mul(f32, glam::Mat3) -> glam::Mat3);
impl_op!(Mul::mul(f32, glam::Mat4) -> glam::Mat4);
impl_op!(Mul::mul(glam::Mat2, glam::Vec2) -> glam::Vec2);
impl_op!(Mul::mul(glam::Mat3, glam::Vec3) -> glam::Vec3);
impl_op!(Mul::mul(glam::Mat4, glam::Vec4) -> glam::Vec4);
