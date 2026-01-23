use {
    crate::{
        irc::{Comp, Fields, GroupMember, Input, InputKind, Stage},
        module::{self, BuildFn, Format, GroupFormat, Info, Make, Module, VertexFormat},
    },
    glam::{IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4},
    std::{any::TypeId, fmt, marker::PhantomData},
};

pub struct RenderInput<V, I>(V, I);

pub struct Render<I, S> {
    module: Module,
    input: PhantomData<I>,
    set: PhantomData<S>,
}

impl<I, S> Render<I, S> {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            input: PhantomData,
            set: PhantomData,
        }
    }

    pub fn module(self) -> Module {
        self.module
    }

    pub fn debug(&self) -> impl fmt::Display {
        #[cfg(feature = "wgsl")]
        {
            fmt::from_fn(|f| {
                let info = &self.module.info;
                let mut count = 0;
                for vertex in info.vertex() {
                    writeln!(f, "// vert  {count}: {vertex:?}")?;
                    count += 1;
                }

                for instance in info.instance() {
                    writeln!(f, "// inst  {count}: {instance:?}")?;
                    count += 1;
                }

                count = 0;
                for group in info.groups() {
                    write!(f, "// group {count}: ")?;
                    count += 1;
                    for stage in group.stages() {
                        write!(f, "{stage:?} ")?;
                    }

                    writeln!(f)?;
                    for (bind_count, bind) in group.bindings().enumerate() {
                        writeln!(f, "// \tbind {bind_count}: {bind:?}")?;
                    }
                }

                write!(f, "\n{}", self.module.wgsl)?;
                Ok(())
            })
        }

        #[cfg(not(feature = "wgsl"))]
        {
            "(module)"
        }
    }
}

pub const fn render() -> MakeRender {
    let empty = GroupEntry {
        id: TypeId::of::<()>(),
        format: &[],
    };

    MakeRender {
        vertex: None,
        instance: None,
        groups: [empty; _],
        groups_len: 0,
    }
}

#[derive(Clone, Copy)]
struct VertexEntry {
    id: TypeId,
    format: &'static [VertexFormat],
}

#[derive(Clone, Copy)]
struct GroupEntry {
    id: TypeId,
    format: &'static [GroupFormat],
}

pub struct MakeRender {
    vertex: Option<VertexEntry>,
    instance: Option<VertexEntry>,
    groups: [GroupEntry; Info::MAX_GROUPS],
    groups_len: usize,
}

impl MakeRender {
    pub const fn with_vertex<V>(mut self) -> Self
    where
        V: Vertex + 'static,
    {
        self.vertex = Some(VertexEntry {
            id: TypeId::of::<V>(),
            format: V::FORMAT,
        });

        self
    }

    pub const fn with_instance<I>(mut self) -> Self
    where
        I: Vertex + 'static,
    {
        self.instance = Some(VertexEntry {
            id: TypeId::of::<I>(),
            format: I::FORMAT,
        });

        self
    }

    pub const fn with_group<G>(mut self) -> Self
    where
        G: Group + 'static,
    {
        assert!(self.groups_len != Info::MAX_GROUPS, "too many groups");
        self.groups[self.groups_len] = GroupEntry {
            id: TypeId::of::<G>(),
            format: G::FORMAT,
        };

        self.groups_len += 1;
        self
    }

    fn groups(&self) -> &[GroupEntry] {
        &self.groups[..self.groups_len]
    }

    fn are_groups_unique(&self) -> bool {
        let mut groups = self.groups;
        groups[..self.groups_len].sort_unstable_by_key(|g| g.id);
        groups[..self.groups_len]
            .windows(2)
            .all(|w| if let [a, b] = w { a.id != b.id } else { true })
    }
}

pub struct Kind {
    input: InputKind,
    id: TypeId,
}

impl Kind {
    const fn new(input: InputKind, id: TypeId) -> Self {
        Self { input, id }
    }
}

pub const fn func<S>(stage: Stage, build: BuildFn) -> Function
where
    S: KindsOf,
{
    Function {
        stage,
        build,
        kinds: S::KINDS,
    }
}

pub struct Function {
    stage: Stage,
    build: BuildFn,
    kinds: &'static [Kind],
}

pub fn type_check(render: MakeRender, fns: &[Function]) -> impl FnOnce() -> Comp<Module> {
    assert!(render.are_groups_unique(), "groups must be unique");

    let mut vs = None;
    let mut fs = None;
    let mut groups = [TypeId::of::<()>(); Info::MAX_GROUPS];
    let mut groups_len = 0;
    let mut group_stages = [0; _];

    for func in fns {
        match func.stage {
            Stage::Regular => {}
            Stage::Vertex => {
                assert!(vs.is_none(), "should be only one vertex shader");
                vs = Some(func);
            }
            Stage::Fragment => {
                assert!(fs.is_none(), "should be only one fragment shader");
                fs = Some(func);
            }
        }
    }

    let vs = vs.expect("no vertex shader");
    let fs = fs.expect("no fragment shader");

    let vertex_index = vs
        .kinds
        .iter()
        .position(|k| Some(k.id) == render.vertex.map(|v| v.id));

    let instance_index = vs
        .kinds
        .iter()
        .position(|k| Some(k.id) == render.instance.map(|v| v.id));

    for Kind { input, id } in vs.kinds {
        if let InputKind::Group = input {
            let Some(group) = groups.get_mut(groups_len) else {
                panic!("too many groups");
            };

            *group = *id;
            group_stages[groups_len] |= Stage::Vertex as u8;
            groups_len += 1;
        }
    }

    for Kind { input, id } in fs.kinds {
        if let InputKind::Group = input {
            if let Some(index) = groups[..groups_len].iter().position(|g| g == id) {
                group_stages[index] |= Stage::Fragment as u8;
                continue;
            }

            let Some(group) = groups.get_mut(groups_len) else {
                panic!("too many groups");
            };

            *group = *id;
            group_stages[groups_len] |= Stage::Fragment as u8;
            groups_len += 1;
        }
    }

    assert!(
        groups[..groups_len]
            .iter()
            .eq(render.groups().iter().map(|g| &g.id)),
        "group types do not match",
    );

    move || {
        let info = Info {
            vertex: render.vertex.map(|v| v.format).unwrap_or_default(),
            instance: render.instance.map(|v| v.format).unwrap_or_default(),
            groups: render.groups.map(|g| g.format),
            group_stages,
        };

        let mut ms = [
            Make::vertex(vs.build, vertex_index, instance_index),
            Make::fragment(fs.build),
        ];

        module::make(info, &mut ms)
    }
}

pub trait KindsOf {
    const KINDS: &[Kind];
}

macro_rules! kinds {
    ($($ty:ident)*) => {
        impl<R, $($ty),*> KindsOf for fn($($ty),*) -> R
        where
            $(
                $ty: Input + 'static,
            )*
        {
            const KINDS: &[Kind] = &[
                $(Kind::new($ty::KIND, TypeId::of::<$ty>())),*
            ];
        }
    };
}

kinds!();
kinds!(A);
kinds!(A B);
kinds!(A B C);
kinds!(A B C D);
kinds!(A B C D E);
kinds!(A B C D E F);
kinds!(A B C D E F G);
kinds!(A B C D E F G H);
kinds!(A B C D E F G H I);

pub trait VertexMember {
    const FORMAT: VertexFormat;
}

impl VertexMember for f32 {
    const FORMAT: VertexFormat = VertexFormat::Scalar(Format::Float32);
}

impl VertexMember for i32 {
    const FORMAT: VertexFormat = VertexFormat::Scalar(Format::Sint32);
}

impl VertexMember for u32 {
    const FORMAT: VertexFormat = VertexFormat::Scalar(Format::Uint32);
}

impl VertexMember for Vec2 {
    const FORMAT: VertexFormat = VertexFormat::Vec2(Format::Float32);
}

impl VertexMember for Vec3 {
    const FORMAT: VertexFormat = VertexFormat::Vec3(Format::Float32);
}

impl VertexMember for Vec4 {
    const FORMAT: VertexFormat = VertexFormat::Vec4(Format::Float32);
}

impl VertexMember for IVec2 {
    const FORMAT: VertexFormat = VertexFormat::Vec2(Format::Sint32);
}

impl VertexMember for IVec3 {
    const FORMAT: VertexFormat = VertexFormat::Vec3(Format::Sint32);
}

impl VertexMember for IVec4 {
    const FORMAT: VertexFormat = VertexFormat::Vec4(Format::Sint32);
}

impl VertexMember for UVec2 {
    const FORMAT: VertexFormat = VertexFormat::Vec2(Format::Uint32);
}

impl VertexMember for UVec3 {
    const FORMAT: VertexFormat = VertexFormat::Vec3(Format::Uint32);
}

impl VertexMember for UVec4 {
    const FORMAT: VertexFormat = VertexFormat::Vec4(Format::Uint32);
}

pub trait Vertex {
    const FORMAT: &[VertexFormat];
}

impl<V> Vertex for V
where
    V: Fields<Tuple: VertexMembers>,
{
    const FORMAT: &[VertexFormat] = V::Tuple::FORMAT;
}

pub trait Group {
    const FORMAT: &[GroupFormat];
}

impl<G> Group for G
where
    G: Fields<Tuple: GroupMembers>,
{
    const FORMAT: &[GroupFormat] = G::Tuple::FORMAT;
}

pub trait VertexMembers {
    const FORMAT: &[VertexFormat];
}

pub trait GroupMembers {
    const FORMAT: &[GroupFormat];
}

macro_rules! members {
    ($($ty:ident)*) => {
        impl<$($ty),*> VertexMembers for ($($ty,)*)
        where
            $(
                $ty: VertexMember,
            )*
        {
            const FORMAT: &[VertexFormat] = &[
                $($ty::FORMAT,)*
            ];
        }

        impl<$($ty),*> GroupMembers for ($($ty,)*)
        where
            $(
                $ty: GroupMember,
            )*
        {
            const FORMAT: &[GroupFormat] = &[
                $($ty::FORMAT,)*
            ];
        }
    };
}

members!(A);
members!(A B);
members!(A B C);
members!(A B C D);
members!(A B C D E);
members!(A B C D E F);
