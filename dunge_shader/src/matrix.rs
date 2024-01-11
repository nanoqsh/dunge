use crate::{
    eval::{Eval, Expr, Exprs, GetEntry},
    types::{self, Matrix},
};

macro_rules! impl_eval_mat {
    ($g:ty => $t:ty) => {
        impl<E> Eval<E> for $g
        where
            E: GetEntry,
        {
            type Out = $t;

            fn eval(self, en: &mut E) -> Expr {
                let mut components = Vec::with_capacity(<$t>::TYPE.dims());
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
