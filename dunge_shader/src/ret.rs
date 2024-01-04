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
            type Output = Ret<Binary<Ret<A, $a>, $b>, $r>;

            fn $f(self, b: $a) -> Self::Output {
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

impl<A, O> ops::Mul<f32> for Ret<A, O>
where
    O: types::Vector<Scalar = f32>,
{
    type Output = Ret<Binary<Self, f32>, O>;

    fn mul(self, b: f32) -> Self::Output {
        Ret::new(Binary {
            a: self,
            b,
            op: Op::Mul,
        })
    }
}

impl<A, O> ops::Mul<Ret<A, O>> for f32
where
    O: types::Vector<Scalar = Self>,
{
    type Output = Ret<Binary<Self, Ret<A, O>>, O>;

    fn mul(self, b: Ret<A, O>) -> Self::Output {
        Ret::new(Binary {
            a: self,
            b,
            op: Op::Mul,
        })
    }
}

impl<A, B, O> ops::Add<Ret<B, O>> for Ret<A, O>
where
    O: types::Vector,
{
    type Output = Ret<Binary<Self, Ret<B, O>>, O>;

    fn add(self, b: Ret<B, O>) -> Self::Output {
        Ret::new(Binary {
            a: self,
            b,
            op: Op::Add,
        })
    }
}

impl<A, B, O> ops::Sub<Ret<B, O>> for Ret<A, O>
where
    O: types::Vector,
{
    type Output = Ret<Binary<Self, Ret<B, O>>, O>;

    fn sub(self, b: Ret<B, O>) -> Self::Output {
        Ret::new(Binary {
            a: self,
            b,
            op: Op::Sub,
        })
    }
}
