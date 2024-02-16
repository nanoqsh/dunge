use {
    crate::{
        eval::{Branch, Eval, Expr, GetEntry},
        ret::Ret,
        types,
    },
    std::marker::PhantomData,
};

pub fn if_then_else<C, A, B, X, Y, E>(c: C, a: A, b: B) -> Ret<IfThenElse<C, A, B, E>, X::Out>
where
    C: Eval<E, Out = bool>,
    A: FnOnce() -> X,
    B: FnOnce() -> Y,
    X: Eval<E>,
    X::Out: types::Value,
    Y: Eval<E, Out = X::Out>,
{
    Ret::new(IfThenElse {
        c,
        a,
        b,
        e: PhantomData,
    })
}

pub struct IfThenElse<C, A, B, E> {
    c: C,
    a: A,
    b: B,
    e: PhantomData<E>,
}

impl<C, A, B, X, Y, E> Eval<E> for Ret<IfThenElse<C, A, B, E>, X::Out>
where
    C: Eval<E>,
    A: FnOnce() -> X,
    B: FnOnce() -> Y,
    X: Eval<E>,
    X::Out: types::Value,
    Y: Eval<E>,
    E: GetEntry,
{
    type Out = X::Out;

    fn eval(self, en: &mut E) -> Expr {
        let IfThenElse { c, a, b, .. } = self.get();
        let c = c.eval(en);
        let a = |en: &mut E| a().eval(en);
        let b = |en: &mut E| Some(b().eval(en));
        let valty = <X::Out as types::Value>::VALUE_TYPE;
        let ty = en.get_entry().new_type(valty.ty());
        let branch = Branch::new(en.get_entry(), ty);
        branch.add(en, c, a, b);
        branch.load(en.get_entry())
    }
}

pub fn default<B, Y, E>(expr: B) -> Else<B>
where
    B: FnOnce() -> Y,
    Y: Eval<E>,
{
    Else(expr)
}

pub struct Else<B>(B);

impl<B> Else<B> {
    pub fn when<C, A, X, Y, E>(self, cond: C, expr: A) -> Ret<When<C, A, B, E>, X::Out>
    where
        C: Eval<E, Out = bool>,
        A: FnOnce() -> X,
        B: FnOnce() -> Y,
        X: Eval<E>,
        X::Out: types::Value,
        Y: Eval<E, Out = X::Out>,
    {
        Ret::new(When {
            c: cond,
            a: expr,
            b: self.0,
            e: PhantomData,
        })
    }
}

pub struct When<C, A, B, E> {
    c: C,
    a: A,
    b: B,
    e: PhantomData<E>,
}

impl<C, A, B, E, O> Ret<When<C, A, B, E>, O> {
    #[allow(clippy::type_complexity)]
    pub fn when<D, F, Z>(self, cond: D, expr: F) -> Ret<When<C, A, When<D, F, B, E>, E>, O>
    where
        D: Eval<E, Out = bool>,
        F: FnOnce() -> Z,
        Z: Eval<E, Out = O>,
    {
        let when = self.get();
        Ret::new(When {
            c: when.c,
            a: when.a,
            b: When {
                c: cond,
                a: expr,
                b: when.b,
                e: PhantomData,
            },
            e: PhantomData,
        })
    }
}

impl<C, A, B, X, E> Eval<E> for Ret<When<C, A, B, E>, X::Out>
where
    C: Eval<E>,
    A: FnOnce() -> X,
    B: EvalBranch<E>,
    X: Eval<E>,
    X::Out: types::Value,
    E: GetEntry,
{
    type Out = X::Out;

    fn eval(self, en: &mut E) -> Expr {
        let when = self.get();
        let valty = <X::Out as types::Value>::VALUE_TYPE;
        let ty = en.get_entry().new_type(valty.ty());
        let branch = Branch::new(en.get_entry(), ty);
        when.eval_else(en, &branch);
        branch.load(en.get_entry())
    }
}

pub trait EvalBranch<E> {
    fn eval_else(self, en: &mut E, branch: &Branch) -> Option<Expr>;
}

impl<F, R, E> EvalBranch<E> for F
where
    F: FnOnce() -> R,
    R: Eval<E>,
{
    fn eval_else(self, en: &mut E, _: &Branch) -> Option<Expr> {
        Some(self().eval(en))
    }
}

impl<C, A, B, X, E> EvalBranch<E> for When<C, A, B, E>
where
    C: Eval<E>,
    A: FnOnce() -> X,
    X: Eval<E>,
    B: EvalBranch<E>,
    E: GetEntry,
{
    fn eval_else(self, en: &mut E, branch: &Branch) -> Option<Expr> {
        let Self { c, a, b, .. } = self;
        let c = c.eval(en);
        let a = |en: &mut E| a().eval(en);
        let b = |en: &mut E| b.eval_else(en, branch);
        branch.add(en, c, a, b);
        None
    }
}
