//! Shader binding types.

use {
    crate::{
        GroupLegacy,
        buffer::{self, Sampler, Texture, TextureSampler},
        group::{BoundTexture, Take},
        shader::{Shader, ShaderData},
        state::State,
        store::{Storage, Uniform},
        store_old,
    },
    std::{cell, iter, marker::PhantomData, sync::Arc},
};

pub trait Visit {
    const N_MEMBERS: usize = 1;
    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>);
}

impl<V> Visit for &V
where
    V: Visit,
{
    const N_MEMBERS: usize = V::N_MEMBERS;

    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>) {
        (*self).visit(visitor);
    }
}

pub struct Visitor<'visit>(Vec<wgpu::BindGroupEntry<'visit>>);

impl<'visit> Visitor<'visit> {
    fn clear(&mut self) {
        self.0.clear();
    }

    fn entries(&self) -> &[wgpu::BindGroupEntry<'visit>] {
        &self.0
    }
}

impl<'visit> Visitor<'visit> {
    fn push(&mut self, resource: wgpu::BindingResource<'visit>) {
        let binding = self.0.len() as u32;
        self.0.push(wgpu::BindGroupEntry { binding, resource });
    }
}

impl<V> Visit for store_old::StorageOld<V>
where
    V: ?Sized,
{
    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(wgpu::BindingResource::Buffer(binding));
    }
}

impl<V> Visit for store_old::UniformOld<V> {
    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(wgpu::BindingResource::Buffer(binding));
    }
}

impl Visit for BoundTexture {
    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>) {
        visitor.push(wgpu::BindingResource::TextureView(&self.0));
    }
}

impl Visit for TextureSampler {
    fn visit<'visit>(&'visit self, visitor: &mut Visitor<'visit>) {
        visitor.push(wgpu::BindingResource::Sampler(self.inner()));
    }
}

pub struct GroupHandler<G, const N: usize> {
    layout: Arc<wgpu::BindGroupLayout>,
    ty: PhantomData<G>,
}

pub(crate) fn update<S, G, const N: usize>(
    state: &State,
    set: &mut UniqueSet<S>,
    handler: &GroupHandler<G::Inner, N>,
    group: G,
) where
    S: Nth<N, Output = G::Inner>,
    G: Group,
{
    let group = group.groups()[N];
    let mut entries = Entries::new();
    group.group(&mut entries);

    let desc = wgpu::BindGroupDescriptor {
        label: None,
        layout: &handler.layout,
        entries: &entries.entries,
    };

    set.bind_groups()[N] = state.device().create_bind_group(&desc);
}

pub struct GroupHandlerOld<S, P> {
    id: usize,
    layout: Arc<wgpu::BindGroupLayout>,
    ty: PhantomData<(S, P)>,
}

pub trait Bind<S> {
    fn bind(&self) -> Bindings<'_>;
}

impl<S, B> Bind<S> for &B
where
    B: Bind<S>,
{
    fn bind(&self) -> Bindings<'_> {
        (**self).bind()
    }
}

impl<S, B> Bind<S> for cell::Ref<'_, B>
where
    B: Bind<S>,
{
    fn bind(&self) -> Bindings<'_> {
        (**self).bind()
    }
}

pub struct Bindings<'group> {
    pub(crate) bind_groups: &'group [wgpu::BindGroup],
}

pub(crate) fn update_old<S, G>(
    state: &State,
    set: &mut UniqueSet<S>,
    handler: &GroupHandlerOld<S, G::Projection>,
    group: G,
) where
    G: Visit + GroupLegacy,
{
    let device = state.device();
    group.set(|_, visitor| {
        let entries = visitor.entries();
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &handler.layout,
            entries,
        };

        let new = device.create_bind_group(&desc);
        let groups = set.bind_groups();
        groups[handler.id] = new;
    });
}

pub struct UniqueSet<S>(SharedSet<S>);

impl<S> UniqueSet<S> {
    pub(crate) fn new<G, const N: usize>(state: &State, shader: &ShaderData, groups: G) -> Self
    where
        G: Groups<N, Inner = S>,
    {
        let bind_groups = make(state, shader, &groups.groups());
        Self(SharedSet {
            bind_groups,
            ty: PhantomData,
        })
    }

    pub(crate) fn from_data<D>(state: &State, shader: &ShaderData, set: D) -> Self
    where
        D: Data<Set = S>,
    {
        let groups = shader.groups();
        let mut bind_groups = Vec::with_capacity(groups.len());

        let device = state.device();
        set.set(|id, visitor| {
            let layout = &groups[id];
            let entries = visitor.entries();
            let desc = wgpu::BindGroupDescriptor {
                label: None,
                layout,
                entries,
            };

            bind_groups.push(device.create_bind_group(&desc));
        });

        Self(SharedSet {
            bind_groups: Arc::from(bind_groups),
            ty: PhantomData,
        })
    }

    pub fn shared(self) -> SharedSet<S> {
        self.0
    }

    pub fn handler<I, const N: usize>(&self, shader: &Shader<I, S>) -> GroupHandler<S::Output, N>
    where
        S: Nth<N>,
    {
        GroupHandler {
            layout: shader.data().groups()[N].clone(),
            ty: PhantomData,
        }
    }

    pub fn handler_old<K>(&self, shader: &Shader<K, S>) -> GroupHandlerOld<S, S::Projection>
    where
        S: Take<0>,
    {
        self.handler_n_old(shader)
    }

    fn handler_n_old<K, const N: usize>(
        &self,
        shader: &Shader<K, S>,
    ) -> GroupHandlerOld<S, S::Projection>
    where
        S: Take<N>,
    {
        let groups = shader.data().groups();
        let layout = Arc::clone(&groups[N]);

        GroupHandlerOld {
            id: N,
            layout,
            ty: PhantomData,
        }
    }

    fn bind_groups(&mut self) -> &mut [wgpu::BindGroup] {
        Arc::get_mut(&mut self.0.bind_groups).expect("uniqueness is guaranteed by the type")
    }
}

impl<S> Bind<S> for UniqueSet<S> {
    fn bind(&self) -> Bindings<'_> {
        self.0.bind()
    }
}

fn make(state: &State, shader: &ShaderData, set: &[&dyn Group]) -> Arc<[wgpu::BindGroup]> {
    let groups = shader.groups();
    let mut bind_groups = Vec::with_capacity(groups.len());

    let mut entries = Entries::new();
    for (layout, group) in iter::zip(groups, set) {
        entries.clear();
        group.group(&mut entries);

        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries.entries,
        };

        bind_groups.push(state.device().create_bind_group(&desc));
    }

    Arc::from(bind_groups)
}

#[derive(Clone)]
pub struct SharedSet<S> {
    bind_groups: Arc<[wgpu::BindGroup]>,
    ty: PhantomData<S>,
}

impl<S> Bind<S> for SharedSet<S> {
    fn bind(&self) -> Bindings<'_> {
        Bindings {
            bind_groups: &self.bind_groups,
        }
    }
}

pub trait Group {
    type Inner
    where
        Self: Sized;

    fn group<'group>(&'group self, e: &mut Entries<'group>);
}

impl<G> Group for &G
where
    G: Group,
{
    type Inner = G::Inner;

    fn group<'group>(&'group self, e: &mut Entries<'group>) {
        (**self).group(e);
    }
}

impl<V> Group for Uniform<V> {
    type Inner = Self;

    fn group<'group>(&'group self, e: &mut Entries<'group>) {
        e.add_buffer(self.data().buffer().as_entire_buffer_binding());
    }
}

impl<V> Group for Storage<V>
where
    V: ?Sized,
{
    type Inner = Self;

    fn group<'group>(&'group self, e: &mut Entries<'group>) {
        e.add_buffer(self.data().buffer().as_entire_buffer_binding());
    }
}

impl<S, const D: usize> Group for Texture<S, D> {
    type Inner = Self;

    fn group<'group>(&'group self, e: &mut Entries<'group>) {
        e.add_texture(buffer::view(self));
    }
}

impl Group for Sampler {
    type Inner = Self;

    fn group<'group>(&'group self, e: &mut Entries<'group>) {
        e.add_sampler(self.inner().downcast_ref().expect("sampler"));
    }
}

pub struct Entries<'group> {
    binding: u32,
    entries: Vec<wgpu::BindGroupEntry<'group>>,
}

impl<'group> Entries<'group> {
    fn new() -> Self {
        Self {
            binding: 0,
            entries: Vec::with_capacity(4),
        }
    }

    fn clear(&mut self) {
        self.binding = 0;
        self.entries.clear();
    }

    fn bind(&mut self) -> u32 {
        let binding = self.binding;
        self.binding += 1;
        binding
    }

    fn add_buffer(&mut self, buffer: wgpu::BufferBinding<'group>) {
        let binding = self.bind();
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(buffer),
        });
    }

    fn add_texture(&mut self, view: &'group wgpu::TextureView) {
        let binding = self.bind();
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        });
    }

    fn add_sampler(&mut self, sampler: &'group wgpu::Sampler) {
        let binding = self.bind();
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        });
    }
}

pub trait Groups<const N: usize> {
    type Inner;
    fn groups(&self) -> [&dyn Group; N];
}

impl<G> Groups<1> for G
where
    G: Group,
{
    type Inner = (G::Inner,);

    fn groups(&self) -> [&dyn Group; 1] {
        [self]
    }
}

impl<A> Groups<1> for (A,)
where
    A: Group,
{
    type Inner = (A::Inner,);

    fn groups(&self) -> [&dyn Group; 1] {
        [&self.0]
    }
}

impl<A, B> Groups<2> for (A, B)
where
    A: Group,
    B: Group,
{
    type Inner = (A::Inner, B::Inner);

    fn groups(&self) -> [&dyn Group; 2] {
        [&self.0, &self.1]
    }
}

impl<A, B, C> Groups<3> for (A, B, C)
where
    A: Group,
    B: Group,
    C: Group,
{
    type Inner = (A::Inner, B::Inner, C::Inner);

    fn groups(&self) -> [&dyn Group; 3] {
        [&self.0, &self.1, &self.2]
    }
}

impl<A, B, C, D> Groups<4> for (A, B, C, D)
where
    A: Group,
    B: Group,
    C: Group,
    D: Group,
{
    type Inner = (A::Inner, B::Inner, C::Inner, D::Inner);

    fn groups(&self) -> [&dyn Group; 4] {
        [&self.0, &self.1, &self.2, &self.3]
    }
}

impl<A, B, C, D, E> Groups<5> for (A, B, C, D, E)
where
    A: Group,
    B: Group,
    C: Group,
    D: Group,
    E: Group,
{
    type Inner = (A::Inner, B::Inner, C::Inner, D::Inner, E::Inner);

    fn groups(&self) -> [&dyn Group; 5] {
        [&self.0, &self.1, &self.2, &self.3, &self.4]
    }
}

impl<A, B, C, D, E, F> Groups<6> for (A, B, C, D, E, F)
where
    A: Group,
    B: Group,
    C: Group,
    D: Group,
    E: Group,
    F: Group,
{
    type Inner = (A::Inner, B::Inner, C::Inner, D::Inner, E::Inner, F::Inner);

    fn groups(&self) -> [&dyn Group; 6] {
        [&self.0, &self.1, &self.2, &self.3, &self.4, &self.5]
    }
}

pub trait Nth<const N: usize> {
    type Output;
}

impl<A> Nth<0> for (A,) {
    type Output = A;
}

impl<A, B> Nth<0> for (A, B) {
    type Output = A;
}

impl<A, B> Nth<1> for (A, B) {
    type Output = B;
}

impl<A, B, C> Nth<0> for (A, B, C) {
    type Output = A;
}

impl<A, B, C> Nth<1> for (A, B, C) {
    type Output = B;
}

impl<A, B, C> Nth<2> for (A, B, C) {
    type Output = C;
}

impl<A, B, C, D> Nth<0> for (A, B, C, D) {
    type Output = A;
}

impl<A, B, C, D> Nth<1> for (A, B, C, D) {
    type Output = B;
}

impl<A, B, C, D> Nth<2> for (A, B, C, D) {
    type Output = C;
}

impl<A, B, C, D> Nth<3> for (A, B, C, D) {
    type Output = D;
}

impl<A, B, C, D, E> Nth<0> for (A, B, C, D, E) {
    type Output = A;
}

impl<A, B, C, D, E> Nth<1> for (A, B, C, D, E) {
    type Output = B;
}

impl<A, B, C, D, E> Nth<2> for (A, B, C, D, E) {
    type Output = C;
}

impl<A, B, C, D, E> Nth<3> for (A, B, C, D, E) {
    type Output = D;
}

impl<A, B, C, D, E> Nth<4> for (A, B, C, D, E) {
    type Output = E;
}

impl<A, B, C, D, E, F> Nth<0> for (A, B, C, D, E, F) {
    type Output = A;
}

impl<A, B, C, D, E, F> Nth<1> for (A, B, C, D, E, F) {
    type Output = B;
}

impl<A, B, C, D, E, F> Nth<2> for (A, B, C, D, E, F) {
    type Output = C;
}

impl<A, B, C, D, E, F> Nth<3> for (A, B, C, D, E, F) {
    type Output = D;
}

impl<A, B, C, D, E, F> Nth<4> for (A, B, C, D, E, F) {
    type Output = E;
}

impl<A, B, C, D, E, F> Nth<5> for (A, B, C, D, E, F) {
    type Output = F;
}

pub trait Data {
    type Set;

    fn set<F>(&self, f: F)
    where
        F: FnMut(usize, &Visitor<'_>);
}

impl<G> Data for G
where
    G: Visit + GroupLegacy,
{
    type Set = (G::Projection,);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let mut visitor = Visitor(Vec::with_capacity(G::N_MEMBERS));
        self.visit(&mut visitor);
        f(0, &visitor);
    }
}

impl<A> Data for (A,)
where
    A: Visit + GroupLegacy,
{
    type Set = (A::Projection,);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let mut visitor = Visitor(Vec::with_capacity(A::N_MEMBERS));
        self.0.visit(&mut visitor);
        f(0, &visitor);
    }
}

impl<A, B> Data for (A, B)
where
    A: Visit + GroupLegacy,
    B: Visit + GroupLegacy,
{
    type Set = (A::Projection, B::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(A::N_MEMBERS, B::N_MEMBERS);
        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);
    }
}

impl<A, B, C> Data for (A, B, C)
where
    A: Visit + GroupLegacy,
    B: Visit + GroupLegacy,
    C: Visit + GroupLegacy,
{
    type Set = (A::Projection, B::Projection, C::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(A::N_MEMBERS, usize::max(B::N_MEMBERS, C::N_MEMBERS));
        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);

        visitor.clear();
        self.2.visit(&mut visitor);
        f(2, &visitor);
    }
}

impl<A, B, C, D> Data for (A, B, C, D)
where
    A: Visit + GroupLegacy,
    B: Visit + GroupLegacy,
    C: Visit + GroupLegacy,
    D: Visit + GroupLegacy,
{
    type Set = (A::Projection, B::Projection, C::Projection, D::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(
            usize::max(A::N_MEMBERS, B::N_MEMBERS),
            usize::max(C::N_MEMBERS, D::N_MEMBERS),
        );

        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);

        visitor.clear();
        self.2.visit(&mut visitor);
        f(2, &visitor);

        visitor.clear();
        self.3.visit(&mut visitor);
        f(3, &visitor);
    }
}
