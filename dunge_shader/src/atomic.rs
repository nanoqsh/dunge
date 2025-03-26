use crate::{
    op::Ret,
    sl::{Eval, GetEntry},
    types::{Array, Atomic, StorageReadWrite},
};

pub struct AtomicOp<A, B, C> {
    op: naga::AtomicFunction,
    arr: A,
    idx: B,
    incr: C,
}

fn atomic_op<A, B, C>(
    arr: A,
    idx: B,
    incr: C,
    op: naga::AtomicFunction,
) -> Ret<AtomicOp<A, B, C>, u32> {
    let a = AtomicOp { op, arr, idx, incr };

    Ret::new(a)
}

pub fn atomic_add<A, B, C, E>(arr: A, idx: B, incr: C) -> Ret<AtomicOp<A, B, C>, u32>
where
    A: Eval<E, Out = Array<Atomic<u32>, StorageReadWrite>>,
    B: Eval<E, Out = u32>,
    C: Eval<E, Out = u32>,
{
    atomic_op(arr, idx, incr, naga::AtomicFunction::Add)
}

impl<A, B, C, E> Eval<E> for Ret<AtomicOp<A, B, C>, u32>
where
    E: GetEntry,
    A: Eval<E, Out = Array<Atomic<u32>, StorageReadWrite>>,
    B: Eval<E, Out = u32>,
    C: Eval<E, Out = u32>,
{
    type Out = u32;

    fn eval(self, en: &mut E) -> crate::sl::Expr {
        let me = self.get();
        let arr = me.arr.eval(en);
        let idx = me.idx.eval(en);
        let access = en.get_entry().access(arr, idx);
        let val = me.incr.eval(en);

        let ty = <u32 as crate::types::Value>::VALUE_TYPE.ty();
        let ty = en.get_entry().new_type(ty);
        en.get_entry().atomic(access, val, me.op, ty)
    }
}
