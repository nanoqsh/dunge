use std::marker::PhantomData;

use crate::{
    op::Ret,
    sl::{Eval, Expr},
};

/// Helper trait implemented by the `DynLander` which is the expression
/// type just inside the type-erased `DynRet` expression. `EvalBoxed` is a dyn-compatible
/// and the `eval_boxed` method is dyn-dispatchable.
trait EvalBoxed<E> {
    type Out;

    /// Helper method required for "dyn" expressions whose
    /// type depends on runtime information.  Do not implement
    /// or call this method.  Should only be used within dyn_expr.
    /// The &mut self will effectively be consumed by this call,
    /// leaving some default Self value behind.
    fn eval_boxed(&mut self, en: &mut E) -> Expr;
}

struct DynLander<T, O> {
    a: Option<T>,
    phantom: PhantomData<O>,
}

impl<E, T, O> EvalBoxed<E> for DynLander<T, O>
where
    T: Eval<E>,
{
    type Out = O;

    fn eval_boxed(&mut self, en: &mut E) -> crate::sl::Expr
    where
        T: Eval<E>,
    {
        let mut new_self = DynLander {
            a: None,
            phantom: PhantomData,
        };
        std::mem::swap(&mut new_self, self);
        new_self.a.expect("DynLander should not be None").eval(en)
    }
}
pub struct DynRet<E, O> {
    a: Box<dyn EvalBoxed<E, Out = O>>,
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
