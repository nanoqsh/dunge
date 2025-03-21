use std::marker::PhantomData;

use crate::{
    op::Ret,
    sl::{Eval, Expr},
};

struct DynLander<T, O> {
    a: Option<T>,
    phantom: PhantomData<O>,
}

impl<E, T, O> Eval<E> for DynLander<T, O>
where
    T: Eval<E>,
{
    type Out = O;

    fn eval(self, en: &mut E) -> crate::sl::Expr {
        self.a.unwrap().eval(en)
    }

    fn eval_boxed(&mut self, en: &mut E) -> crate::sl::Expr
    where
        T: Eval<E>,
    {
        let mut new_self = DynLander {
            a: None,
            phantom: PhantomData,
        };
        std::mem::swap(&mut new_self, self);
        new_self.eval(en)
    }
}
pub struct DynRet<E, O> {
    a: Box<dyn Eval<E, Out = O>>,
}

impl<E, O> DynRet<E, O> {
    fn new<P>(a: Ret<P, O>) -> DynRet<E, O>
    where
        Ret<P, O>: Eval<E, Out = O>,
        P: 'static,
        O: 'static,
    {
        let lander: DynLander<Ret<P, O>, O> = DynLander {
            a: Some(a),
            phantom: PhantomData,
        };
        DynRet {
            a: Box::new(lander),
        }
    }
}

/// Make a "dyn" or "type erased" expression from a concrete
/// expression. This is useful if you want to change the shader
/// definition based on runtime information.
pub fn dyn_expr<P, E, O>(a: Ret<P, O>) -> Ret<DynRet<E, O>, O>
where
    Ret<P, O>: Eval<E, Out = O>,
    P: 'static,
    O: 'static,
{
    Ret::new(DynRet::new(a))
}

impl<E, O> Eval<E> for Ret<DynRet<E, O>, O> {
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        self.get().a.eval_boxed(en)
    }
}
