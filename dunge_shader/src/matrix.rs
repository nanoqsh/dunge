use {
    crate::{
        access::{Access, Dimension},
        eval::{Eval, EvalTuple, Evaluated, Expr, Exprs, GetEntry},
        op::Ret,
        types::{self, Matrix},
    },
    std::marker::PhantomData,
};

macro_rules! impl_eval_mat {
    ($g:ty => $t:ty) => {
        impl<E> Eval<E> for $g
        where
            E: GetEntry,
        {
            type Out = $t;

            fn eval(self, en: &mut E) -> Expr {
                let mut components = Vec::with_capacity(<$t>::TYPE.dims() as usize);
                self.into_matrix(|vector| {
                    let v = vector.eval(en).get();
                    components.push(v);
                });

                let en = en.get_entry();
                let ty = en.new_type(<$t>::TYPE.ty());
                en.compose(ty, Exprs(components))
            }
        }
    };
}

impl_eval_mat!(glam::Mat2 => types::Mat2);
impl_eval_mat!(glam::Mat3 => types::Mat3);
impl_eval_mat!(glam::Mat4 => types::Mat4);

impl Access for types::Mat2 {
    type Dimension = Dimension<2>;
    type Member = types::Vec2<f32>;
}

impl Access for types::Mat3 {
    type Dimension = Dimension<3>;
    type Member = types::Vec3<f32>;
}

impl Access for types::Mat4 {
    type Dimension = Dimension<4>;
    type Member = types::Vec4<f32>;
}

type Matrix2<X, Y, E> = Ret<NewMat<(X, Y), E>, types::Mat2>;

pub const fn mat2<X, Y, E>(x: X, y: Y) -> Matrix2<X, Y, E>
where
    X: Eval<E, Out = types::Vec2<f32>>,
    Y: Eval<E, Out = types::Vec2<f32>>,
{
    Ret::new(NewMat::new((x, y)))
}

type Matrix3<X, Y, Z, E> = Ret<NewMat<(X, Y, Z), E>, types::Mat3>;

pub const fn mat3<X, Y, Z, E>(x: X, y: Y, z: Z) -> Matrix3<X, Y, Z, E>
where
    X: Eval<E, Out = types::Vec3<f32>>,
    Y: Eval<E, Out = types::Vec3<f32>>,
    Z: Eval<E, Out = types::Vec3<f32>>,
{
    Ret::new(NewMat::new((x, y, z)))
}

type Matrix4<X, Y, Z, W, E> = Ret<NewMat<(X, Y, Z, W), E>, types::Mat4>;

pub const fn mat4<X, Y, Z, W, E>(x: X, y: Y, z: Z, w: W) -> Matrix4<X, Y, Z, W, E>
where
    X: Eval<E, Out = types::Vec4<f32>>,
    Y: Eval<E, Out = types::Vec4<f32>>,
    Z: Eval<E, Out = types::Vec4<f32>>,
    W: Eval<E, Out = types::Vec4<f32>>,
{
    Ret::new(NewMat::new((x, y, z, w)))
}

pub struct NewMat<A, E> {
    a: A,
    e: PhantomData<E>,
}

impl<A, E> NewMat<A, E> {
    const fn new(a: A) -> Self {
        Self { a, e: PhantomData }
    }
}

impl<A, O, E> Eval<E> for Ret<NewMat<A, E>, O>
where
    A: EvalTuple<E>,
    O: Matrix,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        self.get().a.eval(en, &mut o);
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        let components = o.into_iter().collect();
        en.compose(ty, components)
    }
}

trait IntoMatrix {
    type Vector;

    fn into_matrix<F>(self, f: F)
    where
        F: FnMut(Self::Vector);
}

impl IntoMatrix for glam::Mat2 {
    type Vector = glam::Vec2;

    fn into_matrix<F>(self, mut f: F)
    where
        F: FnMut(Self::Vector),
    {
        self.to_cols_array_2d().map(|v| f(Self::Vector::from(v)));
    }
}

impl IntoMatrix for glam::Mat3 {
    type Vector = glam::Vec3;

    fn into_matrix<F>(self, mut f: F)
    where
        F: FnMut(Self::Vector),
    {
        self.to_cols_array_2d().map(|v| f(Self::Vector::from(v)));
    }
}

impl IntoMatrix for glam::Mat4 {
    type Vector = glam::Vec4;

    fn into_matrix<F>(self, mut f: F)
    where
        F: FnMut(Self::Vector),
    {
        self.to_cols_array_2d().map(|v| f(Self::Vector::from(v)));
    }
}
