use {
    crate::{
        set::{Bind, Bindings},
        workload::Workload,
    },
    std::{iter, marker::PhantomData},
};

pub struct Compute<'com> {
    pub(crate) pass: wgpu::ComputePass<'com>,
}

impl<'com> Compute<'com> {
    #[inline]
    pub fn workload<I>(&mut self, workload: &Workload<I>) -> On<'com, '_, I, state::Workload> {
        let mut on = On::new(Runner {
            pass: &mut self.pass,
        });

        on.run.workload(workload.compute());
        on
    }
}

pub struct Input<S>(S);

pub trait Types {
    type Set;
}

impl<S> Types for Input<S> {
    type Set = S;
}

pub mod state {
    pub enum Workload {}
    pub enum Set {}
    pub enum Dispatch {}
}

#[diagnostic::on_unimplemented(
    message = "Compute cannot transition from `{A}` to `{B}` state",
    label = "This compute function cannot be called"
)]
pub trait To<A, B> {}

impl<S> To<state::Workload, state::Set> for Input<S> {}
impl To<state::Workload, state::Dispatch> for Input<()> {}

impl<S> To<state::Set, state::Dispatch> for Input<S> {}

impl<S> To<state::Dispatch, state::Workload> for Input<S> {}
impl<S> To<state::Dispatch, state::Set> for Input<S> {}
impl<S> To<state::Dispatch, state::Dispatch> for Input<S> {}

struct Runner<'com, 'work> {
    pass: &'work mut wgpu::ComputePass<'com>,
}

impl Runner<'_, '_> {
    #[inline]
    fn workload(&mut self, compute: &wgpu::ComputePipeline) {
        self.pass.set_pipeline(compute);
    }

    #[inline]
    fn set(&mut self, bindings: Bindings<'_>) {
        for (id, group) in iter::zip(0.., bindings.bind_groups) {
            self.pass.set_bind_group(id, group, &[]);
        }
    }

    #[inline]
    fn dispatch(&mut self, x: u32, y: u32, z: u32) {
        self.pass.dispatch_workgroups(x, y, z);
    }
}

pub struct On<'com, 'work, I, A> {
    run: Runner<'com, 'work>,
    inp: PhantomData<(I, A)>,
}

impl<'com, 'work, I, A> On<'com, 'work, I, A> {
    #[inline]
    fn new(run: Runner<'com, 'work>) -> Self {
        Self {
            run,
            inp: PhantomData,
        }
    }

    #[inline]
    fn to<B>(self) -> On<'com, 'work, I, B>
    where
        I: To<A, B>,
    {
        On {
            run: self.run,
            inp: PhantomData,
        }
    }

    #[inline]
    pub fn workload(mut self, workload: &Workload<I>) -> On<'com, 'work, I, state::Workload>
    where
        I: To<A, state::Workload>,
    {
        self.run.workload(workload.compute());
        self.to()
    }

    #[inline]
    pub fn set<S>(mut self, set: &S) -> On<'com, 'work, I, state::Set>
    where
        I: To<A, state::Set> + Types,
        S: Bind<I::Set>,
    {
        self.run.set(set.bind());
        self.to()
    }

    #[inline]
    pub fn dispatch(mut self, x: u32, y: u32, z: u32) -> On<'com, 'work, I, state::Dispatch>
    where
        I: To<A, state::Dispatch>,
    {
        self.run.dispatch(x, y, z);
        self.to()
    }
}
