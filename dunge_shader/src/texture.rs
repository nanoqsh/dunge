use crate::{
    eval::{Eval, Expr, Fs, GetEntry},
    op::Ret,
    types,
};

type Tex<T, S, C, O> = Ret<Samp<T, S, C>, types::Vec4<O>>;

/// Performs the [`textureSample`](https://www.w3.org/TR/WGSL/#texturesample) function.
pub const fn texture_sample<T, S, C>(tex: T, sam: S, crd: C) -> Tex<T, S, C, f32>
where
    T: Eval<Fs, Out = types::Texture2d<f32>>,
    S: Eval<Fs, Out = types::Sampler>,
    C: Eval<Fs, Out = types::Vec2<f32>>,
{
    Ret::new(Samp { tex, sam, crd })
}

pub struct Samp<T, S, C> {
    tex: T,
    sam: S,
    crd: C,
}

impl<T, S, C, F> Eval<Fs> for Ret<Samp<T, S, C>, types::Vec4<F>>
where
    T: Eval<Fs, Out = types::Texture2d<F>>,
    S: Eval<Fs, Out = types::Sampler>,
    C: Eval<Fs, Out = types::Vec2<f32>>,
{
    type Out = types::Vec4<F>;

    fn eval(self, en: &mut Fs) -> Expr {
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
    pub(crate) fn expr(self) -> naga::Expression {
        naga::Expression::ImageSample {
            image: self.tex.get(),
            sampler: self.sam.get(),
            gather: None,
            coordinate: self.crd.get(),
            array_index: None,
            offset: None,
            level: naga::SampleLevel::Auto,
            depth_ref: None,
        }
    }
}
