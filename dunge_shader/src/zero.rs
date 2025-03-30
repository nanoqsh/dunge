use crate::{
    eval::{Eval, Expr, GetEntry},
    op::Ret,
    types,
};

pub fn zero_value<O>() -> Ret<Zero, O>
where
    O: types::Value,
{
    Ret::new(Zero(()))
}

pub struct Zero(());

impl<O, E> Eval<E> for Ret<Zero, O>
where
    O: types::Value,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let en = en.get_entry();
        let ty = O::VALUE_TYPE.ty(en);
        en.zero_value(ty)
    }
}
