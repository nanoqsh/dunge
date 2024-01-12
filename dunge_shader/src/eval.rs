use {
    crate::{
        context::{Context, Info, InputInfo, Stages},
        define::Define,
        module::{Module, Out, Output},
        ret::Ret,
        types::{self, MemberType, ScalarType, VectorType},
    },
    naga::{
        AddressSpace, Arena, BinaryOperator, Binding, Block, BuiltIn, EntryPoint, Expression,
        Function, FunctionArgument, FunctionResult, GlobalVariable, Handle, Literal, Range,
        ResourceBinding, SampleLevel, ShaderStage, Span, Statement, StructMember, Type, TypeInner,
        UniqueArena,
    },
    std::{array, cell::Cell, collections::HashMap, iter, mem, rc::Rc},
};

pub(crate) fn make<O>(cx: Context, output: O) -> Module
where
    O: Output,
{
    let Out { place, color } = output.output();
    let mut compl = Compiler::default();
    let make_input = |info| match info {
        InputInfo::Vert(Info { def, .. }) | InputInfo::Inst(Info { def, .. }) => {
            let mut new = def.into_iter().map(Member::from_vecty);
            Argument::from_type(compl.define_input(&mut new))
        }
        InputInfo::Index => Argument {
            ty: compl.define_index(),
            binding: Some(Binding::BuiltIn(BuiltIn::VertexIndex)),
        },
    };

    let inputs: Vec<_> = cx.inputs.iter().copied().map(make_input).collect();
    for (id, en) in iter::zip(0.., &cx.groups) {
        compl.define_group(id, en.def());
    }

    let (fs, required, fsty) = {
        let mut fs = Fs::new(compl);
        let ex = color.eval(&mut fs);
        fs.inner.ret(ex);
        let fsty = fs.define_fragment_ty();
        let mut args = [fsty].into_iter().map(Argument::from_type);
        let built = fs.inner.build(Stage::Fragment, &mut args, Return::Color);
        (built, fs.required, fsty)
    };

    let vs = {
        let mut vs = Vs::new(fs.compl);
        let ex = place.eval(&mut vs);
        let eval = |req: Required| match req.evalf {
            EvalFunction::Position => ex,
            EvalFunction::Fn(f) => f(&mut vs),
        };

        let out = required.into_iter().map(eval).collect();
        let res = vs.inner.compose(fsty, out);
        vs.inner.ret(res);
        let mut args = inputs.into_iter();
        vs.inner.build(Stage::Vertex, &mut args, Return::Ty(fsty))
    };

    let compl = vs.compl;
    let nm = naga::Module {
        types: compl.types,
        global_variables: compl.globs.vars,
        entry_points: vec![vs.point, fs.point],
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    {
        use naga::valid::{Capabilities, ValidationFlags, Validator};

        let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
        if let Err(err) = validator.validate(&nm) {
            log::error!("{nm:#?}");
            panic!("shader error: {err}\n{val:#?}", val = err.as_inner());
        }
    }

    Module { cx, nm }
}

#[derive(Clone, Copy)]
pub struct Expr(Handle<Expression>);

impl Expr {
    pub(crate) fn get(self) -> Handle<Expression> {
        self.0
    }
}

pub(crate) struct Exprs(pub Vec<Handle<Expression>>);

impl FromIterator<Expr> for Exprs {
    fn from_iter<T: IntoIterator<Item = Expr>>(iter: T) -> Self {
        Self(iter.into_iter().map(Expr::get).collect())
    }
}

pub trait Eval<E>: Sized {
    type Out;
    fn eval(self, en: &mut E) -> Expr;
}

impl<E> Eval<E> for f32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::F32(self))
    }
}

impl<E> Eval<E> for i32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::I32(self))
    }
}

impl<E> Eval<E> for u32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::U32(self))
    }
}

impl<E> Eval<E> for bool
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::Bool(self))
    }
}

#[derive(Clone, Copy)]
pub struct ReadIndex {
    id: u32,
}

impl ReadIndex {
    pub(crate) const fn new(id: u32) -> Ret<Self, u32> {
        Ret::new(Self { id })
    }
}

impl Eval<Vs> for Ret<ReadIndex, u32> {
    type Out = u32;

    fn eval(self, en: &mut Vs) -> Expr {
        en.inner.argument(self.get().id)
    }
}

#[derive(Clone, Copy)]
pub struct ReadInput {
    id: u32,
    index: u32,
}

impl ReadInput {
    pub const fn new<T>(id: u32, index: u32) -> Ret<Self, T> {
        Ret::new(Self { id, index })
    }
}

impl<T> Eval<Vs> for Ret<ReadInput, T> {
    type Out = T;

    fn eval(self, en: &mut Vs) -> Expr {
        let en = &mut en.inner;
        let arg = en.argument(self.get().id);
        en.access_index(arg, self.get().index)
    }
}

#[derive(Clone, Default)]
pub struct GlobalOut(Rc<Cell<Stages>>);

impl GlobalOut {
    fn with_stage(&self, stage: Stage) {
        self.0.set(self.0.get().with(stage));
    }

    pub fn get(&self) -> Stages {
        self.0.get()
    }
}

pub struct ReadGlobal {
    id: u32,
    binding: u32,
    is_value: bool,
    out: GlobalOut,
}

impl ReadGlobal {
    pub const fn new<T>(id: u32, binding: u32, is_value: bool, out: GlobalOut) -> Ret<Self, T> {
        Ret::new(Self {
            id,
            binding,
            is_value,
            out,
        })
    }
}

impl<T, E> Eval<E> for Ret<ReadGlobal, T>
where
    E: GetEntry,
{
    type Out = T;

    fn eval(self, en: &mut E) -> Expr {
        let ReadGlobal {
            id,
            binding,
            is_value,
            out,
        } = self.get();

        out.with_stage(E::STAGE);
        let en = en.get_entry();
        let res = ResourceBinding { group: id, binding };
        let var = en.compl.globs.get(&res);
        let global = en.global(var);
        if is_value {
            en.load(global)
        } else {
            global
        }
    }
}

pub const fn fragment<A>(a: A) -> Ret<Fragment<A>, A::Out>
where
    A: Eval<Vs>,
    A::Out: types::Vector,
{
    Ret::new(Fragment(a))
}

pub struct Fragment<A>(A);

impl<A> Eval<Fs> for Ret<Fragment<A>, A::Out>
where
    A: Eval<Vs> + 'static,
    A::Out: types::Vector,
{
    type Out = A::Out;

    fn eval(self, en: &mut Fs) -> Expr {
        let vecty = <A::Out as types::Vector>::TYPE;
        let index = en.push_evalf(vecty, |en| self.get().0.eval(en));
        let en = &mut en.inner;
        let arg = en.argument(0);
        en.access_index(arg, index)
    }
}

pub fn thunk<A, E, const N: usize>(a: A) -> [Ret<Thunk<A>, A::Out>; N]
where
    A: Eval<E>,
{
    let state = State::Eval(a);
    let inner = Rc::new(Cell::new(state));
    array::from_fn(|_| Ret::new(Thunk(Rc::clone(&inner))))
}

pub struct Thunk<A>(Rc<Cell<State<A>>>);

impl<A, O, E> Eval<E> for Ret<Thunk<A>, O>
where
    A: Eval<E>,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Thunk(state) = self.get();
        match state.replace(State::None) {
            State::None => unreachable!(),
            State::Eval(a) => {
                let ex = a.eval(en);
                state.set(State::Expr(ex));
                ex
            }
            State::Expr(ex) => ex,
        }
    }
}

enum State<A> {
    None,
    Eval(A),
    Expr(Expr),
}

#[derive(Default)]
pub(crate) struct Evaluated([Option<Expr>; 4]);

impl Evaluated {
    fn push(&mut self, expr: Expr) {
        let slot = self
            .0
            .iter_mut()
            .find(|slot| slot.is_none())
            .expect("empty slot");

        *slot = Some(expr);
    }

    pub fn into_iter(self) -> impl Iterator<Item = Expr> {
        self.0.into_iter().flatten()
    }
}

pub(crate) trait EvalTuple<E> {
    fn eval(self, en: &mut E, o: &mut Evaluated);
}

macro_rules! impl_eval_tuple {
    ($($t:ident),*) => {
        impl<$($t),*, E> EvalTuple<E> for ($($t),*,)
        where
            $(
                $t: Eval<E>,
            )*
        {
            #[allow(non_snake_case)]
            fn eval(self, en: &mut E, o: &mut Evaluated) {
                let ($($t),*,) = self;
                $(
                    o.push($t.eval(en));
                )*
            }
        }
    };
}

impl_eval_tuple!(X);
impl_eval_tuple!(X, Y);
impl_eval_tuple!(X, Y, Z);
impl_eval_tuple!(X, Y, Z, W);

#[derive(Clone, Copy)]
pub(crate) enum Stage {
    Vertex,
    Fragment,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Vertex => "vs",
            Self::Fragment => "fs",
        }
    }

    fn shader_stage(self) -> ShaderStage {
        match self {
            Self::Vertex => ShaderStage::Vertex,
            Self::Fragment => ShaderStage::Fragment,
        }
    }
}

pub(crate) trait GetEntry {
    const STAGE: Stage;
    fn get_entry(&mut self) -> &mut Entry;
}

pub struct Vs {
    inner: Entry,
}

impl Vs {
    fn new(compl: Compiler) -> Self {
        Self {
            inner: Entry::new(compl),
        }
    }
}

impl GetEntry for Vs {
    const STAGE: Stage = Stage::Vertex;

    fn get_entry(&mut self) -> &mut Entry {
        &mut self.inner
    }
}

struct Member {
    vecty: VectorType,
    built: Option<BuiltIn>,
}

impl Member {
    fn from_vecty(vecty: VectorType) -> Self {
        Self { vecty, built: None }
    }
}

enum EvalFunction {
    Position,
    Fn(Box<dyn FnOnce(&mut Vs) -> Expr>),
}

struct Required {
    vecty: VectorType,
    evalf: EvalFunction,
}

pub struct Fs {
    inner: Entry,
    required: Vec<Required>,
}

impl Fs {
    fn new(compl: Compiler) -> Self {
        Self {
            inner: Entry::new(compl),
            required: vec![Required {
                vecty: VectorType::Vec4f,
                evalf: EvalFunction::Position,
            }],
        }
    }

    fn push_evalf<F>(&mut self, vecty: VectorType, f: F) -> u32
    where
        F: FnOnce(&mut Vs) -> Expr + 'static,
    {
        let req = Required {
            vecty,
            evalf: EvalFunction::Fn(Box::new(f)),
        };

        let index = self.required.len();
        self.required.push(req);
        index as u32
    }

    fn define_fragment_ty(&mut self) -> Handle<Type> {
        let member = |req: &Required| match req.evalf {
            EvalFunction::Position => Member {
                vecty: req.vecty,
                built: Some(BuiltIn::Position { invariant: false }),
            },
            EvalFunction::Fn(_) => Member::from_vecty(req.vecty),
        };

        let mut members = self.required.iter().map(member);
        self.inner.compl.define_input(&mut members)
    }
}

impl GetEntry for Fs {
    const STAGE: Stage = Stage::Fragment;

    fn get_entry(&mut self) -> &mut Entry {
        &mut self.inner
    }
}

struct Built {
    compl: Compiler,
    point: EntryPoint,
}

#[derive(Clone, Copy)]
enum Return {
    Ty(Handle<Type>),
    Color,
}

type Args<'a> = dyn Iterator<Item = Argument> + 'a;

struct Argument {
    ty: Handle<Type>,
    binding: Option<Binding>,
}

impl Argument {
    fn from_type(ty: Handle<Type>) -> Self {
        Self { ty, binding: None }
    }

    fn into_function(self) -> FunctionArgument {
        FunctionArgument {
            name: None,
            ty: self.ty,
            binding: self.binding,
        }
    }
}

pub(crate) enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl Op {
    fn operator(self) -> BinaryOperator {
        match self {
            Self::Add => BinaryOperator::Add,
            Self::Sub => BinaryOperator::Subtract,
            Self::Mul => BinaryOperator::Multiply,
            Self::Div => BinaryOperator::Divide,
            Self::Rem => BinaryOperator::Modulo,
        }
    }
}

pub(crate) enum Func {
    Cos,
    Cosh,
    Sin,
    Sinh,
    Tan,
    Tanh,
}

impl Func {
    fn expr(self, ev: Evaluated) -> Expression {
        use naga::MathFunction;

        let fun = match self {
            Self::Cos => MathFunction::Cos,
            Self::Cosh => MathFunction::Cosh,
            Self::Sin => MathFunction::Sin,
            Self::Sinh => MathFunction::Sinh,
            Self::Tan => MathFunction::Tan,
            Self::Tanh => MathFunction::Tanh,
        };

        let mut exprs = ev.into_iter().map(Expr::get);
        Expression::Math {
            fun,
            arg: exprs.next().expect("first argument"),
            arg1: exprs.next(),
            arg2: exprs.next(),
            arg3: exprs.next(),
        }
    }
}

pub(crate) struct Sampled {
    pub tex: Expr,
    pub sam: Expr,
    pub crd: Expr,
}

impl Sampled {
    fn expr(self) -> Expression {
        Expression::ImageSample {
            image: self.tex.0,
            sampler: self.sam.0,
            gather: None,
            coordinate: self.crd.0,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
        }
    }
}

pub struct Entry {
    compl: Compiler,
    exprs: Arena<Expression>,
    stats: Statements,
}

impl Entry {
    fn new(compl: Compiler) -> Self {
        Self {
            compl,
            exprs: Arena::default(),
            stats: Statements::default(),
        }
    }

    pub(crate) fn new_type(&mut self, ty: Type) -> Handle<Type> {
        self.compl.types.insert(ty, Span::UNDEFINED)
    }

    fn literal(&mut self, literal: Literal) -> Expr {
        let ex = Expression::Literal(literal);
        Expr(self.exprs.append(ex, Span::UNDEFINED))
    }

    fn argument(&mut self, n: u32) -> Expr {
        let ex = Expression::FunctionArgument(n);
        Expr(self.exprs.append(ex, Span::UNDEFINED))
    }

    fn global(&mut self, var: Handle<GlobalVariable>) -> Expr {
        let ex = Expression::GlobalVariable(var);
        Expr(self.exprs.append(ex, Span::UNDEFINED))
    }

    fn load(&mut self, ptr: Expr) -> Expr {
        let ex = Expression::Load { pointer: ptr.0 };
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    fn access_index(&mut self, base: Expr, index: u32) -> Expr {
        let ex = Expression::AccessIndex {
            base: base.0,
            index,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn convert(&mut self, expr: Expr, ty: ScalarType) -> Expr {
        let (kind, width) = ty.inner();
        let ex = Expression::As {
            expr: expr.0,
            kind,
            convert: Some(width),
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn binary(&mut self, op: Op, a: Expr, b: Expr) -> Expr {
        let ex = Expression::Binary {
            op: op.operator(),
            left: a.0,
            right: b.0,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn math(&mut self, f: Func, exprs: Evaluated) -> Expr {
        let ex = f.expr(exprs);
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn compose(&mut self, ty: Handle<Type>, exprs: Exprs) -> Expr {
        let ex = Expression::Compose {
            ty,
            components: exprs.0,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn sample(&mut self, ex: Sampled) -> Expr {
        let handle = self.exprs.append(ex.expr(), Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        Expr(handle)
    }

    fn ret(&mut self, value: Expr) {
        let st = Statement::Return {
            value: Some(value.0),
        };

        self.stats.push(st, &self.exprs);
    }

    fn build(mut self, stage: Stage, args: &mut Args, ret: Return) -> Built {
        const COLOR_TYPE: Type = VectorType::Vec4f.ty();

        let res = match ret {
            Return::Ty(ty) => FunctionResult { ty, binding: None },
            Return::Color => FunctionResult {
                ty: self.new_type(COLOR_TYPE),
                binding: Some(binding_location(0, &COLOR_TYPE.inner)),
            },
        };

        let point = EntryPoint {
            name: stage.name().to_owned(),
            stage: stage.shader_stage(),
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: Function {
                arguments: args.map(Argument::into_function).collect(),
                result: Some(res),
                expressions: self.exprs,
                body: Block::from_vec(self.stats.0),
                ..Default::default()
            },
        };

        Built {
            compl: self.compl,
            point,
        }
    }
}

#[derive(Default)]
struct Statements(Vec<Statement>);

impl Statements {
    fn push(&mut self, st: Statement, exprs: &Arena<Expression>) {
        if let Statement::Emit(new) = &st {
            if let Some(Statement::Emit(top)) = self.0.last_mut() {
                let top_range = top.zero_based_index_range();
                let new_range = new.zero_based_index_range();
                if top_range.end == new_range.start {
                    let merged = top_range.start..new_range.end;
                    *top = Range::from_zero_based_index_range(merged, exprs);
                    return;
                }
            }
        }

        self.0.push(st);
    }
}

type Members<'a> = dyn ExactSizeIterator<Item = Member> + 'a;

#[derive(Default)]
struct Compiler {
    types: UniqueArena<Type>,
    globs: Globals,
}

impl Compiler {
    fn define_index(&mut self) -> Handle<Type> {
        self.types.insert(ScalarType::Uint.ty(), Span::UNDEFINED)
    }

    fn define_input(&mut self, new: &mut Members) -> Handle<Type> {
        const VECTOR_SIZE: u32 = mem::size_of::<f32>() as u32 * 4;

        let len = new.len();
        let mut members = Vec::with_capacity(len);
        let mut location = 0;
        for (idx, Member { vecty, built }) in iter::zip(0.., new) {
            let ty = vecty.ty();
            let binding = match built {
                Some(bi @ BuiltIn::Position { .. }) => Binding::BuiltIn(bi),
                None => {
                    let curr = location;
                    location += 1;
                    binding_location(curr, &ty.inner)
                }
                _ => unimplemented!(),
            };

            members.push(StructMember {
                name: None,
                ty: self.types.insert(ty, Span::UNDEFINED),
                binding: Some(binding),
                offset: idx * VECTOR_SIZE,
            });
        }

        let ty = Type {
            name: None,
            inner: TypeInner::Struct {
                members,
                span: len as u32 * VECTOR_SIZE,
            },
        };

        self.types.insert(ty, Span::UNDEFINED)
    }

    fn define_group(&mut self, group: u32, def: Define<MemberType>) {
        for (binding, member) in iter::zip(0.., def) {
            let space = member.address_space();
            let ty = self.types.insert(member.ty(), Span::UNDEFINED);
            let res = ResourceBinding { group, binding };
            self.globs.add(space, ty, res);
        }
    }
}

#[derive(Default)]
struct Globals {
    vars: Arena<GlobalVariable>,
    handles: HashMap<ResourceBinding, Handle<GlobalVariable>>,
}

impl Globals {
    fn add(&mut self, space: AddressSpace, ty: Handle<Type>, res: ResourceBinding) {
        self.handles.entry(res.clone()).or_insert_with(|| {
            let var = GlobalVariable {
                name: None,
                space,
                binding: Some(res),
                ty,
                init: None,
            };

            self.vars.append(var, Span::UNDEFINED)
        });
    }

    fn get(&self, res: &ResourceBinding) -> Handle<GlobalVariable> {
        self.handles[res]
    }
}

fn binding_location(location: u32, inner: &TypeInner) -> Binding {
    let mut binding = Binding::Location {
        location,
        second_blend_source: false,
        interpolation: None,
        sampling: None,
    };

    binding.apply_default_interpolation(inner);
    binding
}
