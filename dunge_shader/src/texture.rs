use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        ret::Ret,
        types,
    },
    naga::{Expression, SampleLevel},
    std::marker::PhantomData,
};

type Tex<T, S, C, O, E> = Ret<Samp<T, S, C, E>, types::Vec4<O>>;

pub const fn texture_sample<T, S, C, E>(tex: T, sam: S, crd: C) -> Tex<T, S, C, f32, E>
where
    T: Eval<E, Out = types::Texture2d<f32>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
{
    Ret::new(Samp {
        tex,
        sam,
        crd,
        e: PhantomData,
    })
}

pub struct Samp<T, S, C, E> {
    tex: T,
    sam: S,
    crd: C,
    e: PhantomData<E>,
}

impl<T, S, C, F, E> Eval<E> for Ret<Samp<T, S, C, E>, types::Vec4<F>>
where
    T: Eval<E, Out = types::Texture2d<F>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
    E: GetEntry,
{
    type Out = types::Vec4<F>;

    fn eval(self, en: &mut E) -> Expr {
        let Samp { tex, sam, crd, .. } = self.get();
        let ex = Sampled {
            tex: tex.eval(en),
            sam: sam.eval(en),
            crd: crd.eval(en),
        };

        en.get_entry().sample(ex)
    }
}

pub(crate) struct Sampled {
    tex: Expr,
    sam: Expr,
    crd: Expr,
}

impl Sampled {
    pub fn expr(self) -> Expression {
        Expression::ImageSample {
            image: self.tex.get(),
            sampler: self.sam.get(),
            gather: None,
            coordinate: self.crd.get(),
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
        }
    }
}
