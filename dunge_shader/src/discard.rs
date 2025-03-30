use {
    crate::{
        eval::{Eval, Expr, Fs, GetEntry},
        types,
    },
    std::marker::PhantomData,
};

pub fn discard<O>() -> Discard<O>
where
    O: types::Value,
{
    Discard(PhantomData)
}

pub struct Discard<O>(PhantomData<O>);

impl<O> Eval<Fs> for Discard<O>
where
    O: types::Value,
{
    type Out = O;

    fn eval(self, en: &mut Fs) -> Expr {
        let en = en.get_entry();
        en.kill();

        let ty = O::VALUE_TYPE.ty(en);
        en.zero_value(ty)
    }
}
