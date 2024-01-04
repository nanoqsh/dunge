use crate::{
    eval::{Eval, Expr, GetEntry, Sampled},
    ret::Ret,
    types,
};

type TextureSample<T, S, C, O> = Ret<Sample<T, S, C>, types::Vec4<O>>;

pub const fn texture_sample<T, S, C, E>(tex: T, sam: S, crd: C) -> TextureSample<T, S, C, f32>
where
    T: Eval<E, Out = types::Texture2d<f32>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
{
    Ret::new(Sample { tex, sam, crd })
}

pub struct Sample<T, S, C> {
    tex: T,
    sam: S,
    crd: C,
}

impl<T, S, C, F, E> Eval<E> for Ret<Sample<T, S, C>, types::Vec4<F>>
where
    T: Eval<E, Out = types::Texture2d<F>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
    E: GetEntry,
{
    type Out = types::Vec4<F>;

    fn eval(self, en: &mut E) -> Expr {
        let Sample { tex, sam, crd } = self.get();
        let ex = Sampled {
            tex: tex.eval(en),
            sam: sam.eval(en),
            crd: crd.eval(en),
        };

        en.get_entry().sample(ex)
    }
}
