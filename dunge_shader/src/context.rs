use {
    crate::{
        define::Define,
        eval::{GlobalOut, ReadIndex, Stage},
        group::{self, Group},
        instance::{self, Instance},
        ret::Ret,
        types::{MemberType, VectorType},
        vertex::{self, Vertex},
    },
    std::{any::TypeId, mem, ops},
};

#[derive(Clone, Copy)]
pub struct GroupInfo {
    pub tyid: TypeId,
    pub def: Define<MemberType>,
    pub stages: Stages,
}

#[derive(Clone, Copy, Default)]
pub struct Stages {
    pub vs: bool,
    pub fs: bool,
}

impl Stages {
    pub(crate) fn with(self, stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self { vs: true, ..self },
            Stage::Fragment => Self { fs: true, ..self },
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum InputInfo {
    Vert(VertInfo),
    Inst(InstInfo),
    Index,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct VertInfo {
    pub def: Define<VectorType>,
    pub size: usize,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct InstInfo {
    pub vecty: VectorType,
}

pub(crate) struct GroupEntry {
    tyid: TypeId,
    def: Define<MemberType>,
    out: GlobalOut,
}

impl GroupEntry {
    pub fn def(&self) -> Define<MemberType> {
        self.def
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
                insts: 2,
                group: 4,
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

    fn add_instance(&mut self, vec: VectorType) -> u32 {
        countdown(&mut self.limits.insts, "too many instances in the shader");
        let id = self.inputs.len() as u32;
        let info = InstInfo { vecty: vec };
        self.inputs.push(InputInfo::Inst(info));
        id
    }

    fn add_group(&mut self, tyid: TypeId, def: Define<MemberType>) -> (u32, GlobalOut) {
        countdown(&mut self.limits.group, "too many groups in the shader");
        let out = GlobalOut::default();
        let en = GroupEntry {
            tyid,
            def,
            out: out.clone(),
        };

        let id = self.groups.len() as u32;
        self.groups.push(en);
        (id, out)
    }

    #[doc(hidden)]
    pub fn count_input(&self) -> usize {
        self.inputs
            .iter()
            .filter(|info| matches!(info, InputInfo::Vert(_) | InputInfo::Inst(_)))
            .count()
    }

    #[doc(hidden)]
    pub fn input(&self) -> impl Iterator<Item = InputInfo> + '_ {
        self.inputs.iter().copied()
    }

    #[doc(hidden)]
    pub fn groups(&self) -> impl Iterator<Item = GroupInfo> + '_ {
        self.groups.iter().map(|entry| GroupInfo {
            tyid: entry.tyid,
            def: entry.def,
            stages: entry.out.get(),
        })
    }
}

pub trait FromContextInput {
    type Vertex;
    type Instance;
    fn from_context_input(cx: &mut Context) -> Self;
}

impl<V> FromContextInput for V
where
    V: FromContext,
{
    type Vertex = ();
    type Instance = ();

    fn from_context_input(cx: &mut Context) -> Self {
        V::from_context(cx)
    }
}

pub struct InVertex<V>(V::Projection)
where
    V: Vertex;

impl<V> ops::Deref for InVertex<V>
where
    V: Vertex,
{
    type Target = V::Projection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> FromContextInput for InVertex<V>
where
    V: Vertex,
{
    type Vertex = V;
    type Instance = ();

    fn from_context_input(cx: &mut Context) -> Self {
        let id = cx.add_vertex(V::DEF, mem::size_of::<V>());
        Self(vertex::Projection::projection(id))
    }
}

pub struct InInstance<I>(I::Projection)
where
    I: Instance;

impl<I> ops::Deref for InInstance<I>
where
    I: Instance,
{
    type Target = I::Projection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> FromContextInput for InInstance<I>
where
    I: Instance,
{
    type Vertex = ();
    type Instance = I;

    fn from_context_input(cx: &mut Context) -> Self {
        let mut id = None;
        for vec in I::DEF {
            id.get_or_insert(cx.add_instance(vec));
        }

        let id = id.expect("the instance must have at least one field");
        Self(instance::Projection::projection(id))
    }
}

pub struct In<V, I>(pub V::Projection, pub I::Projection)
where
    V: Vertex,
    I: Instance;

impl<V, I> FromContextInput for In<V, I>
where
    V: Vertex,
    I: Instance,
{
    type Vertex = V;
    type Instance = I;

    fn from_context_input(cx: &mut Context) -> Self {
        let InVertex(vert): InVertex<V> = InVertex::from_context_input(cx);
        let InInstance(inst): InInstance<I> = InInstance::from_context_input(cx);
        Self(vert, inst)
    }
}

pub trait FromContext {
    fn from_context(cx: &mut Context) -> Self;
}

#[derive(Clone, Copy)]
pub struct Index(pub Ret<ReadIndex, u32>);

impl FromContext for Index {
    fn from_context(cx: &mut Context) -> Self {
        let id = cx.add_index();
        Self(ReadIndex::new(id))
    }
}

pub trait ProjectionFromContext {
    type Projection;
    fn from_context(cx: &mut Context) -> Self::Projection;
}

impl ProjectionFromContext for () {
    type Projection = ();
    fn from_context(_: &mut Context) -> Self::Projection {}
}

impl<A> ProjectionFromContext for A
where
    A: Group,
{
    type Projection = A::Projection;

    fn from_context(cx: &mut Context) -> Self::Projection {
        let (id, out) = cx.add_group(TypeId::of::<A::Projection>(), A::DEF);
        group::Projection::projection(id, out)
    }
}

macro_rules! impl_projection_from_context {
    ($($t:ident),*) => {
        impl<$($t),*> ProjectionFromContext for ($($t),*,)
        where
            $(
                $t: Group,
            )*
        {
            type Projection = ($($t::Projection),*,);

            fn from_context(cx: &mut Context) -> Self::Projection {
                (
                    $({
                        let (id, out) = cx.add_group(TypeId::of::<$t::Projection>(), $t::DEF);
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

impl<G> FromContext for Groups<G>
where
    G: ProjectionFromContext,
{
    fn from_context(cx: &mut Context) -> Self {
        Self(G::from_context(cx))
    }
}
