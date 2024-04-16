use {
    crate::eval::{Eval, Expr, GetEntry, Vs},
    naga::{BinaryOperator, UnaryOperator},
    std::{marker::PhantomData, ops},
};

#[must_use]
pub struct Ret<A, O> {
    a: A,
    t: PhantomData<O>,
}

impl<A, T> Ret<A, T> {
    pub(crate) const fn new(a: A) -> Self {
        Self { a, t: PhantomData }
    }

    pub(crate) fn get(self) -> A {
        self.a
    }
}

impl<A, O> Clone for Ret<A, O>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.a.clone())
    }
}

impl<A, O> Copy for Ret<A, O> where A: Copy {}

type Operand<A, B> = Ret<A, <B as Eval<Vs>>::Out>;

pub struct Unary<A> {
    a: A,
    op: Un,
}

impl<A, O, E> Eval<E> for Ret<Unary<A>, O>
where
    A: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Unary { a, op } = self.get();
        let x = a.eval(en);
        en.get_entry().unary(op, x)
    }
}

macro_rules! impl_unary {
    ($o:ident :: $f:ident ( $a:ty ) -> $r:ty) => {
        impl<A> ops::$o for Operand<A, $a> {
            type Output = Operand<Unary<Self>, $r>;

            fn $f(self) -> Self::Output {
                Ret::new(Unary {
                    a: self,
                    op: Un::$o,
                })
            }
        }
    };
}

impl_unary!(Neg::neg(f32) -> f32);
impl_unary!(Neg::neg(glam::Vec2) -> glam::Vec2);
impl_unary!(Neg::neg(glam::Vec3) -> glam::Vec3);
impl_unary!(Neg::neg(glam::Vec4) -> glam::Vec4);
impl_unary!(Neg::neg(glam::Mat2) -> glam::Mat2);
impl_unary!(Neg::neg(glam::Mat3) -> glam::Mat3);
impl_unary!(Neg::neg(glam::Mat4) -> glam::Mat4);

pub struct Binary<A, B> {
    a: A,
    b: B,
    op: Bi,
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

macro_rules! impl_binary {
    ($o:ident :: $f:ident ( $a:ty, $b:ty ) -> $r:ty) => {
        impl<A> ops::$o<Operand<A, $b>> for $a {
            type Output = Operand<Binary<Self, Operand<A, $b>>, $r>;

            fn $f(self, b: Operand<A, $b>) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Bi::$o,
                })
            }
        }

        impl<A> ops::$o<$a> for Operand<A, $b> {
            type Output = Operand<Binary<Self, $a>, $r>;

            fn $f(self, b: $a) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Bi::$o,
                })
            }
        }

        impl<A, B> ops::$o<Operand<B, $b>> for Operand<A, $a> {
            type Output = Operand<Binary<Self, Operand<B, $b>>, $r>;

            fn $f(self, b: Operand<B, $b>) -> Self::Output {
                Ret::new(Binary {
                    a: self,
                    b,
                    op: Bi::$o,
                })
            }
        }
    };
}

impl_binary!(Add::add(f32, f32) -> f32);
impl_binary!(Sub::sub(f32, f32) -> f32);
impl_binary!(Mul::mul(f32, f32) -> f32);
impl_binary!(Div::div(f32, f32) -> f32);
impl_binary!(Rem::rem(f32, f32) -> f32);
impl_binary!(Add::add(i32, i32) -> i32);
impl_binary!(Sub::sub(i32, i32) -> i32);
impl_binary!(Mul::mul(i32, i32) -> i32);
impl_binary!(Div::div(i32, i32) -> i32);
impl_binary!(Rem::rem(i32, i32) -> i32);
impl_binary!(Add::add(u32, u32) -> u32);
impl_binary!(Sub::sub(u32, u32) -> u32);
impl_binary!(Mul::mul(u32, u32) -> u32);
impl_binary!(Div::div(u32, u32) -> u32);
impl_binary!(Rem::rem(u32, u32) -> u32);

impl_binary!(Add::add(glam::Vec2, glam::Vec2) -> glam::Vec2);
impl_binary!(Add::add(glam::Vec3, glam::Vec3) -> glam::Vec3);
impl_binary!(Add::add(glam::Vec4, glam::Vec4) -> glam::Vec4);
impl_binary!(Sub::sub(glam::Vec2, glam::Vec2) -> glam::Vec2);
impl_binary!(Sub::sub(glam::Vec3, glam::Vec3) -> glam::Vec3);
impl_binary!(Sub::sub(glam::Vec4, glam::Vec4) -> glam::Vec4);
impl_binary!(Mul::mul(f32, glam::Vec2) -> glam::Vec2);
impl_binary!(Mul::mul(glam::Vec2, f32) -> glam::Vec2);
impl_binary!(Mul::mul(f32, glam::Vec3) -> glam::Vec3);
impl_binary!(Mul::mul(glam::Vec3, f32) -> glam::Vec3);
impl_binary!(Mul::mul(f32, glam::Vec4) -> glam::Vec4);
impl_binary!(Mul::mul(glam::Vec4, f32) -> glam::Vec4);

impl_binary!(Add::add(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_binary!(Add::add(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_binary!(Add::add(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_binary!(Sub::sub(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_binary!(Sub::sub(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_binary!(Sub::sub(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_binary!(Mul::mul(glam::Mat2, glam::Mat2) -> glam::Mat2);
impl_binary!(Mul::mul(glam::Mat3, glam::Mat3) -> glam::Mat3);
impl_binary!(Mul::mul(glam::Mat4, glam::Mat4) -> glam::Mat4);
impl_binary!(Mul::mul(f32, glam::Mat2) -> glam::Mat2);
impl_binary!(Mul::mul(f32, glam::Mat3) -> glam::Mat3);
impl_binary!(Mul::mul(f32, glam::Mat4) -> glam::Mat4);
impl_binary!(Mul::mul(glam::Mat2, glam::Vec2) -> glam::Vec2);
impl_binary!(Mul::mul(glam::Mat3, glam::Vec3) -> glam::Vec3);
impl_binary!(Mul::mul(glam::Mat4, glam::Vec4) -> glam::Vec4);

pub(crate) enum Un {
    Neg,
}

impl Un {
    pub fn operator(self) -> UnaryOperator {
        match self {
            Self::Neg => UnaryOperator::Negate,
        }
    }
}

pub(crate) enum Bi {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl Bi {
    pub fn operator(self) -> BinaryOperator {
        match self {
            Self::Add => BinaryOperator::Add,
            Self::Sub => BinaryOperator::Subtract,
            Self::Mul => BinaryOperator::Multiply,
            Self::Div => BinaryOperator::Divide,
            Self::Rem => BinaryOperator::Modulo,
        }
    }
}
