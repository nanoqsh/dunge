use {
    crate::{
        define::Define,
        eval::{GlobalOut, ReadIndex},
        group::{self, GroupLegacy},
        instance::{self, Instance},
        module::RenderKind,
        op::Ret,
        stage::Stages,
        types::{MemberData, ValueType, VectorType},
        vertex::{self, Vertex},
    },
    std::iter,
};

#[derive(Clone)]
pub struct GroupInfo {
    pub def: Define<MemberData>,
    pub stages: Stages,
}

#[derive(Clone)]
pub enum InputInfo {
    Vert(VertInfo),
    Inst(InstInfo),
    Index,
    GlobalInvocationId,
}

#[derive(Clone)]
pub struct VertInfo {
    pub def: Define<VectorType>,
    pub size: usize,
}

#[derive(Clone, Copy)]
pub struct InstInfo {
    pub ty: ValueType,
}

pub(crate) struct GroupEntry {
    def: Define<MemberData>,
    out: GlobalOut,
}

impl GroupEntry {
    pub(crate) fn def(&self) -> &Define<MemberData> {
        &self.def
    }
}

struct Limits {
    index: u8,
    verts: u8,
    insts: u8,
    group: u8,
}

fn countdown(v: &mut u8, msg: &str) {
    match v.checked_sub(1) {
        Some(n) => *v = n,
        None => panic!("{msg}"),
    }
}

pub struct Context {
    pub(crate) inputs: Vec<InputInfo>,
    pub(crate) groups: Vec<GroupEntry>,
    limits: Limits,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            inputs: vec![],
            groups: vec![],
            limits: Limits {
                index: 1,
                verts: 1,
                insts: 3,
                group: 1,
            },
        }
    }

    fn add_index(&mut self) -> u32 {
        countdown(&mut self.limits.index, "too many indices in the shader");
        let id = self.inputs.len() as u32;
        self.inputs.push(InputInfo::Index);
        id
    }

    fn add_vertex(&mut self, def: Define<VectorType>, size: usize) -> u32 {
        countdown(&mut self.limits.verts, "too many vertices in the shader");
        let id = self.inputs.len() as u32;
        let info = VertInfo { def, size };
        self.inputs.push(InputInfo::Vert(info));
        id
    }

    fn add_instance(&mut self, ty: ValueType) -> u32 {
        countdown(&mut self.limits.insts, "too many instances in the shader");
        let id = self.inputs.len() as u32;
        let info = InstInfo { ty };
        self.inputs.push(InputInfo::Inst(info));
        id
    }

    fn add_group_set(&mut self) {
        countdown(&mut self.limits.group, "too many groups in the shader");
    }

    fn add_group(&mut self, def: Define<MemberData>) -> (u32, GlobalOut) {
        let out = GlobalOut::default();
        let en = GroupEntry {
            def,
            out: out.clone(),
        };

        let id = self.groups.len() as u32;
        self.groups.push(en);
        (id, out)
    }

    pub(crate) fn into_info(self) -> Info {
        Info {
            inputs: self.inputs,
            groups: self
                .groups
                .into_iter()
                .map(|entry| GroupInfo {
                    def: entry.def,
                    stages: entry.out.get(),
                })
                .collect(),
        }
    }
}

pub struct Info {
    inputs: Vec<InputInfo>,
    groups: Vec<GroupInfo>,
}

impl Info {
    pub fn count_input(&self) -> usize {
        self.inputs
            .iter()
            .filter(|info| matches!(info, InputInfo::Vert(_) | InputInfo::Inst(_)))
            .count()
    }

    pub fn input(&self) -> impl Iterator<Item = &InputInfo> {
        self.inputs.iter()
    }

    pub fn groups(&self) -> impl Iterator<Item = &GroupInfo> {
        self.groups.iter()
    }

    pub fn set_stages(&mut self, stages: &[Stages]) {
        let stages = stages
            .iter()
            .copied()
            .chain(iter::repeat_with(Stages::default));

        for (group, stage) in iter::zip(&mut self.groups, stages) {
            group.stages = stage;
        }
    }
}

pub trait FromRender<K> {
    type Vertex;
    type Instance;
    fn from_render(cx: &mut Context) -> Self;
}

impl<V> FromRender<RenderKind> for V
where
    V: FromContext<RenderKind>,
{
    type Vertex = ();
    type Instance = ();

    fn from_render(cx: &mut Context) -> Self {
        V::from_context(cx)
    }
}

pub struct PassVertex<V>(pub V::Projection)
where
    V: Vertex;

impl<V, O> FromRender<O> for PassVertex<V>
where
    V: Vertex,
{
    type Vertex = V;
    type Instance = ();

    fn from_render(cx: &mut Context) -> Self {
        let id = cx.add_vertex(V::DEF, size_of::<V>());
        Self(vertex::Projection::projection(id))
    }
}

pub struct PassInstance<I>(pub I::Projection)
where
    I: Instance;

impl<I, O> FromRender<O> for PassInstance<I>
where
    I: Instance,
{
    type Vertex = ();
    type Instance = I;

    fn from_render(cx: &mut Context) -> Self {
        let mut id = None;
        for ty in I::DEF.iter() {
            id.get_or_insert(cx.add_instance(ty));
        }

        let id = id.expect("the instance must have at least one field");
        Self(instance::Projection::projection(id))
    }
}

pub struct Pass<V, I>(pub V::Projection, pub I::Projection)
where
    V: Vertex,
    I: Instance;

impl<V, I, O> FromRender<O> for Pass<V, I>
where
    V: Vertex,
    I: Instance,
{
    type Vertex = V;
    type Instance = I;

    fn from_render(cx: &mut Context) -> Self {
        let PassVertex(vert) = <PassVertex<V> as FromRender<O>>::from_render(cx);
        let PassInstance(inst) = <PassInstance<I> as FromRender<O>>::from_render(cx);
        Self(vert, inst)
    }
}

#[derive(Clone, Copy)]
pub struct Index(pub Ret<ReadIndex, u32>);

impl FromContext<RenderKind> for Index {
    type Set = ();

    fn from_context(cx: &mut Context) -> Self {
        let id = cx.add_index();
        Self(ReadIndex::new(id))
    }
}

pub trait ProjectionFromContext {
    type Set;
    type Projection;
    fn from_context(cx: &mut Context) -> Self::Projection;
}

impl ProjectionFromContext for () {
    type Set = Self;
    type Projection = Self;
    fn from_context(_: &mut Context) -> Self::Projection {}
}

impl<A> ProjectionFromContext for A
where
    A: GroupLegacy,
{
    type Set = (A::Projection,);
    type Projection = A::Projection;

    fn from_context(cx: &mut Context) -> Self::Projection {
        cx.add_group_set();
        let (id, out) = cx.add_group(A::DEF);
        group::Projection::projection(id, out)
    }
}

macro_rules! impl_projection_from_context {
    ($($t:ident),*) => {
        impl<$($t),*> ProjectionFromContext for ($($t),*,)
        where
            $(
                $t: GroupLegacy,
            )*
        {
            type Set = ($($t::Projection),*,);
            type Projection = ($($t::Projection),*,);

            fn from_context(cx: &mut Context) -> Self::Projection {
                cx.add_group_set();

                (
                    $({
                        let (id, out) = cx.add_group($t::DEF);
                        group::Projection::projection(id, out)
                    }),*,
                )
            }
        }
    };
}

impl_projection_from_context!(A);
impl_projection_from_context!(A, B);
impl_projection_from_context!(A, B, C);
impl_projection_from_context!(A, B, C, D);

pub struct Groups<G>(pub G::Projection)
where
    G: ProjectionFromContext;

impl<G, K> FromContext<K> for Groups<G>
where
    G: ProjectionFromContext,
{
    type Set = G::Set;

    fn from_context(cx: &mut Context) -> Self {
        Self(G::from_context(cx))
    }
}

pub trait FromContext<K> {
    type Set;
    fn from_context(cx: &mut Context) -> Self;
}

pub trait TakeSet {
    type Set;
}

impl TakeSet for ((), (), ()) {
    type Set = ();
}

impl<A> TakeSet for ((A,), (), ()) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((A, B), (), ()) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((A, B, C), (), ()) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((A, B, C, D), (), ()) {
    type Set = (A, B, C, D);
}

impl<A> TakeSet for ((), (A,), ()) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((), (A, B), ()) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((), (A, B, C), ()) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((), (A, B, C, D), ()) {
    type Set = (A, B, C, D);
}

impl<A> TakeSet for ((), (), (A,)) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((), (), (A, B)) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((), (), (A, B, C)) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((), (), (A, B, C, D)) {
    type Set = (A, B, C, D);
}
