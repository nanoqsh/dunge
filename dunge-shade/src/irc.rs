use {
    crate::{
        desc::{Sampler, Texture},
        map::{self, Map},
        module::{Boot, GroupFormat, Hook},
        sl,
        store::{Storage, Uniform},
    },
    glam::{IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4},
    naga::{Arena, Handle, Range, Span, Statement, UniqueArena},
    std::{
        any::TypeId,
        error,
        fmt::{self, Write},
        marker::PhantomData,
        num::NonZero,
        ops,
        sync::LazyLock,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Regular = 0,
    Vertex = 1 << 0,
    Fragment = 1 << 1,
}

impl Stage {
    pub(crate) fn from_bits(bits: u8) -> impl Iterator<Item = Self> {
        [Self::Vertex, Self::Fragment]
            .into_iter()
            .filter(move |&s| bits & s as u8 == s as u8)
    }
}

pub const fn inverse_permutation<const N: usize>(indices: [usize; N]) -> [usize; N] {
    let mut perms = [0; N];
    let mut i = 0;
    while i < N {
        perms[indices[i]] = i;
        i += 1;
    }

    perms
}

pub const fn indices<const N: usize>(offsets: [usize; N]) -> [usize; N] {
    let mut indices = [0; N];
    let mut i = 0;
    while i < N {
        let mut s = 0;
        let mut j = 0;
        while j < N {
            s += (offsets[i] > offsets[j]) as usize;
            j += 1;
        }

        indices[i] = s;
        i += 1;
    }

    indices
}

pub struct Fnc<'irc> {
    irc: &'irc mut Irc,
    arguments: Vec<naga::FunctionArgument>,
    result: Option<naga::FunctionResult>,
    local_variables: Arena<naga::LocalVariable>,
    exprs: Arena<naga::Expression>,
    body: Vec<Body>,
}

impl<'irc> Fnc<'irc> {
    pub const fn new(irc: &'irc mut Irc) -> Self {
        Self {
            irc,
            arguments: vec![],
            result: None,
            local_variables: Arena::new(),
            exprs: Arena::new(),
            body: vec![],
        }
    }

    pub(crate) fn irc(&mut self) -> &mut Irc {
        self.irc
    }

    pub fn push(&mut self) {
        self.body.push(Body(vec![]));
    }

    pub fn pop(&mut self) -> Body {
        self.body.pop().expect("pop body")
    }

    pub fn do_point<P, T>(&mut self, value: P) -> Pointer<T>
    where
        P: Reference<T>,
        T: ?Sized,
    {
        match value.reference() {
            TakeReference::Argument(arg) => {
                let ex = self.add_expr(naga::Expression::FunctionArgument(arg.0));
                Pointer::Noop(expr(ex))
            }
            TakeReference::Variable(var) => {
                let pointer = self.add_expr(naga::Expression::LocalVariable(var.0));
                Pointer::Load(expr(pointer))
            }
            TakeReference::GlobalVariable(var) => {
                let pointer = self.add_expr(naga::Expression::GlobalVariable(var.var));
                Pointer::Noop(expr(pointer))
            }
            TakeReference::GlobalVariables(vars) => Pointer::GlobalVariables(vars),
        }
    }

    pub fn do_point_noop<T>(&mut self, ex: Expr<T>) -> Pointer<T> {
        Pointer::Noop(ex)
    }

    pub fn do_value<T>(&mut self, value: T) -> Expr<T>
    where
        T: Value,
    {
        self.irc.add_type(T::NAGA);
        value.expr(self)
    }

    fn do_literal<T>(&mut self, value: T) -> Expr<T>
    where
        T: Scalar,
    {
        self.irc.add_type(<T as Value>::NAGA);
        expr(self.add_expr(naga::Expression::Literal(T::LITERAL(value))))
    }

    fn do_splat<T, I>(&mut self, item: Expr<I>) -> Expr<T>
    where
        T: Value,
    {
        let size = const {
            match T::NAGA {
                Type::Vector { size, .. } => size,
                _ => panic!("value should be a vector"),
            }
        };

        expr(self.add_expr(naga::Expression::Splat {
            size,
            value: item.get(),
        }))
    }

    fn do_math(
        &mut self,
        fun: naga::MathFunction,
        args: &[Handle<naga::Expression>],
    ) -> Handle<naga::Expression> {
        self.add_expr(naga::Expression::Math {
            fun,
            arg: args[0],
            arg1: args.get(1).copied(),
            arg2: args.get(2).copied(),
            arg3: args.get(3).copied(),
        })
    }

    fn do_compose<T, I>(&mut self, components: &[Expr<I>]) -> Expr<T>
    where
        T: Value,
    {
        let ty = self.irc.add_type(T::NAGA);
        expr(self.add_expr(naga::Expression::Compose {
            ty,
            components: components.iter().map(|ex| ex.get()).collect(),
        }))
    }

    pub fn do_compose_array<V, const N: usize>(&mut self, components: [Expr<V>; N]) -> Expr<[V; N]>
    where
        V: Value,
    {
        self.do_compose(&components)
    }

    pub fn do_compose_tuple<T, C, const N: usize>(&mut self, components: C) -> Expr<T>
    where
        T: Value,
        C: ExprTuple<N>,
    {
        let ty = self.irc.add_type(T::NAGA);
        expr(self.add_expr(naga::Expression::Compose {
            ty,
            components: components.get().into(),
        }))
    }

    pub fn do_compose_tuple_with_permutation<T, C, const N: usize>(
        &mut self,
        components: C,
        permutation: [usize; N],
    ) -> Expr<T>
    where
        T: Value,
        C: ExprTuple<N>,
    {
        let ty = self.irc.add_type(T::NAGA);
        let components = components.get();
        expr(self.add_expr(naga::Expression::Compose {
            ty,
            components: permutation.map(|p| components[p]).into(),
        }))
    }

    pub fn do_access_index<B>(
        &mut self,
        base: B,
        index: Expr<u32>,
    ) -> B::Output<<B::Base as Composite>::Output>
    where
        B: BaseAccess<Base: Composite>,
    {
        base.base_index(index, self)
    }

    fn do_index_expr<B, I, O>(&mut self, base: Expr<B>, index: Expr<I>) -> Expr<O>
    where
        B: ?Sized,
    {
        expr(self.add_expr(naga::Expression::Access {
            base: base.get(),
            index: index.get(),
        }))
    }

    fn do_load_by_index<B, I, O>(&mut self, base: Pointer<B>, index: Expr<I>) -> Pointer<O>
    where
        B: ?Sized,
    {
        match base {
            Pointer::Load(ex) | Pointer::Noop(ex) => Pointer::Load(self.do_index_expr(ex, index)),
            Pointer::GlobalVariables(_) => unreachable!(),
        }
    }

    pub fn do_access_field<B, O, F>(&mut self, base: B, f: F) -> B::Output<O>
    where
        B: BaseAccess<Base: Fields>,
        F: FnOnce(<B::Base as Fields>::Fields) -> Access<B::Base, O>,
    {
        base.base_access(f(B::Base::FIELDS), self)
    }

    fn do_access_expr<B, O>(&mut self, base: Expr<B>, access: Access<B, O>) -> Expr<O>
    where
        B: ?Sized,
    {
        expr(self.add_expr(naga::Expression::AccessIndex {
            base: base.get(),
            index: access.index(),
        }))
    }

    fn do_access_pointer<B, O>(&mut self, base: Pointer<B>, access: Access<B, O>) -> Pointer<O>
    where
        B: ?Sized,
    {
        match base {
            Pointer::Load(ex) => Pointer::Load(self.do_access_expr(ex, access)),
            Pointer::Noop(ex) => Pointer::Noop(self.do_access_expr(ex, access)),
            Pointer::GlobalVariables(vars) => {
                let index = access.index() as usize;
                let map = self.irc.global_map.get(&vars).expect("get global map");
                let entry = map[index];
                let pointer = self.add_expr(naga::Expression::GlobalVariable(entry.var));
                Pointer::Noop(expr(pointer))
            }
        }
    }

    fn do_swizzle<B, O>(&mut self, base: Expr<B>, swizzle: Swizzle<B, O>) -> Expr<O> {
        expr(self.add_expr(swizzle.expr(base.get())))
    }

    pub fn do_load<T>(&mut self, pointer: Pointer<T>) -> Expr<T> {
        match pointer {
            Pointer::Load(ex) => expr(self.add_expr(naga::Expression::Load { pointer: ex.get() })),
            Pointer::Noop(ex) => ex,
            Pointer::GlobalVariables(_) => unreachable!(),
        }
    }

    pub fn do_as<F, T>(&mut self, ex: Expr<F>) -> Expr<T>
    where
        T: Scalar,
    {
        self.irc.add_type(<T as Value>::NAGA);
        expr(self.add_expr(naga::Expression::As {
            expr: ex.get(),
            kind: T::NAGA.kind,
            convert: Some(T::NAGA.width),
        }))
    }

    pub fn do_op<T>(&mut self, op: Op<T>) -> Expr<T> {
        expr(self.add_expr(op.0))
    }

    pub fn do_if(&mut self, condition: Expr<bool>, accept: Body, reject: Body) {
        let s = Statement::If {
            condition: condition.get(),
            accept: naga::Block::from_vec(accept.0),
            reject: naga::Block::from_vec(reject.0),
        };

        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    pub fn do_loop(&mut self, body: Body, continuing: Body) {
        let s = Statement::Loop {
            body: naga::Block::from_vec(body.0),
            continuing: naga::Block::from_vec(continuing.0),
            break_if: None,
        };

        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    pub fn do_store<T>(&mut self, pointer: Pointer<T>, value: Expr<T>) {
        let ex = match pointer {
            Pointer::Load(ex) | Pointer::Noop(ex) => ex,
            Pointer::GlobalVariables(_) => unreachable!(),
        };

        let s = Statement::Store {
            pointer: ex.get(),
            value: value.get(),
        };

        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    pub fn do_break(&mut self) {
        let s = Statement::Break;
        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    pub fn do_return<T>(&mut self, value: Expr<T>) {
        let s = Statement::Return {
            value: Some(value.get()),
        };

        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    fn do_discard(&mut self) {
        let s = Statement::Kill;
        self.body.last_mut().expect("top body").push(s, &self.exprs);
    }

    pub fn do_call<T, O, const N: usize>(
        &mut self,
        export: Export<T::Tuple, O>,
        args: T,
    ) -> Comp<Expr<O>>
    where
        T: ExprTuple<N>,
    {
        let build = self
            .irc
            .imports
            .get(export.id)
            .ok_or(FncError::CallImport)?;

        let ex = build(self, Args(&args.get())).map_err(|_| FncError::CallFailed)?;
        Ok(expr(ex))
    }

    pub fn do_method_call<B, O, F>(&mut self, base: Expr<B>, f: F) -> Expr<O>
    where
        B: Methods,
        F: FnOnce(B::Methods) -> Method<B, O>,
    {
        match f(B::METHODS) {
            Method::Swizzle(swizzle) => self.do_swizzle(base, swizzle),
            Method::Expr(e) => e(self, base),
            Method::Load => expr(self.add_expr(naga::Expression::Load {
                pointer: base.get(),
            })),
            Method::Noop => expr(base.get()),
        }
    }

    fn do_image_size<S, O, const D: usize>(&mut self, texture: Expr<Texture<S, D>>) -> Expr<O> {
        expr(self.add_expr(naga::Expression::ImageQuery {
            image: texture.get(),
            query: naga::ImageQuery::Size { level: None },
        }))
    }

    fn do_image_sample<P, const D: usize>(
        &mut self,
        texture: Expr<Texture<f32, D>>,
        sampler: Expr<Sampler>,
        point: Expr<P>,
    ) -> Expr<Vec4> {
        expr(self.add_expr(naga::Expression::ImageSample {
            image: texture.get(),
            sampler: sampler.get(),
            gather: None,
            coordinate: point.get(),
            array_index: None,
            offset: None,
            level: naga::SampleLevel::Auto,
            depth_ref: None,
            clamp_to_edge: false,
        }))
    }

    fn do_image_load<S, P, const D: usize>(
        &mut self,
        texture: Expr<Texture<S, D>>,
        point: Expr<P>,
    ) -> Expr<S::Vec>
    where
        S: Scalar + Dim<4>,
    {
        let zero = 0u32.expr(self);
        expr(self.add_expr(naga::Expression::ImageLoad {
            image: texture.get(),
            coordinate: point.get(),
            array_index: None,
            sample: None,
            level: Some(zero.get()),
        }))
    }

    pub fn add_local<T>(&mut self) -> Variable<T>
    where
        T: Value,
    {
        let ty = self.irc.add_type(T::NAGA);
        let l = naga::LocalVariable {
            name: None,
            ty,
            init: None,
        };

        let var = self.local_variables.append(l, Span::UNDEFINED);
        Variable(var, PhantomData)
    }

    fn add_expr(&mut self, ex: naga::Expression) -> Handle<naga::Expression> {
        let push_emit = !ex.needs_pre_emit();
        let ex = self.exprs.append(ex, Span::UNDEFINED);
        if push_emit {
            let s = naga::Statement::Emit(Range::new_from_bounds(ex, ex));
            self.body.last_mut().expect("top body").push(s, &self.exprs);
        }

        ex
    }

    fn add_argument<T>(&mut self) -> Argument<T>
    where
        T: Value,
    {
        let ty = self.irc.add_type(T::NAGA);
        let argi = self.arguments.len() as u32;
        self.arguments.push(naga::FunctionArgument {
            name: None,
            ty,
            binding: None,
        });

        Argument(argi, PhantomData)
    }

    pub fn input<T>(&mut self) -> impl Reference<T::Ref> + use<T>
    where
        T: Input,
    {
        T::input(self)
    }

    pub fn output<T>(&mut self, stage: Stage)
    where
        T: Output,
    {
        T::output(self, stage);
    }

    fn add_result_type<T>(&mut self, stage: Stage)
    where
        T: Value,
    {
        let binding = 'bind: {
            if !T::NAGA.eq_types(<Vec4 as Value>::NAGA) {
                break 'bind Binding::None;
            }

            if stage == Stage::Vertex {
                Binding::Position
            } else if stage == Stage::Fragment {
                Binding::Location(0)
            } else {
                Binding::None
            }
        };

        // set locations for vertex output
        if stage == Stage::Vertex {
            self.irc.location = Some(0);
        }

        let ty = self.irc.add_type(T::NAGA);
        self.irc.location = None;

        let res = naga::FunctionResult {
            ty,
            binding: binding.naga::<T>(),
        };

        self.result = Some(res);
    }

    pub fn build_group<G>(&mut self) -> GroupBuilder<'_, G>
    where
        G: 'static,
    {
        let id = TypeId::of::<G>();
        let make = match self.irc.group_globals.get(&id) {
            Some(n) => MakeGroup::Cached(*n),
            None => MakeGroup::New(id),
        };

        GroupBuilder {
            irc: self.irc,
            make,
            binding: 0,
            vars: vec![],
            ty: PhantomData,
        }
    }

    pub fn build(self, body: Body, stage: Stage) {
        let f = naga::Function {
            arguments: self.arguments,
            result: self.result,
            local_variables: self.local_variables,
            expressions: self.exprs,
            body: naga::Block::from_vec(body.0),
            ..naga::Function::default()
        };

        let (name, stage) = match stage {
            Stage::Regular => {
                self.irc.functions.append(f, Span::UNDEFINED);
                return;
            }
            Stage::Vertex => ("vs", naga::ShaderStage::Vertex),
            Stage::Fragment => ("fs", naga::ShaderStage::Fragment),
        };

        self.irc.entries.push(naga::EntryPoint {
            name: name.to_owned(),
            stage,
            early_depth_test: None,
            workgroup_size: [0; 3],
            workgroup_size_overrides: None,
            function: f,
            mesh_info: None,
            task_payload: None,
        });
    }
}

#[derive(Debug)]
enum FncError {
    CallImport,
    CallFailed,
}

pub struct Body(Vec<naga::Statement>);

impl Body {
    fn push(&mut self, s: naga::Statement, exprs: &Arena<naga::Expression>) {
        if let Some(naga::Statement::Emit(top)) = self.0.last_mut()
            && let naga::Statement::Emit(new) = &s
            && let top_range = top.index_range()
            && let new_range = new.index_range()
            && top_range.end == new_range.start
        {
            let merged = top_range.start..new_range.end;
            *top = naga::Range::from_index_range(merged, exprs);
            return;
        }

        self.0.push(s);
    }
}

pub trait Function<I, O>: 'static {}
impl<F, O> Function<(), O> for F where F: FnOnce() -> O + 'static {}
impl<F, A, O> Function<(A,), O> for F where F: FnOnce(A) -> O + 'static {}
impl<F, A, B, O> Function<(A, B), O> for F where F: FnOnce(A, B) -> O + 'static {}
impl<F, A, B, C, O> Function<(A, B, C), O> for F where F: FnOnce(A, B, C) -> O + 'static {}
impl<F, A, B, C, D, O> Function<(A, B, C, D), O> for F where F: FnOnce(A, B, C, D) -> O + 'static {}

const fn fnid<F, I, O>(_: F) -> TypeId
where
    F: Function<I, O> + Copy,
{
    TypeId::of::<F>()
}

pub struct Export<I, O> {
    id: TypeId,
    sign: PhantomData<(I, O)>,
}

impl<I, O> Export<I, O> {
    pub fn new<F>(f: F) -> Self
    where
        F: Function<I, O> + Copy,
    {
        Self {
            id: fnid(f),
            sign: PhantomData,
        }
    }
}

struct Math(usize);

impl Math {
    const ABS: Self = Self(0);
    const MIN: Self = Self(1);
    const MAX: Self = Self(2);
    const CLAMP: Self = Self(3);

    const fn get(self) -> usize {
        self.0
    }

    const fn from_usize(n: usize) -> Option<Self> {
        if n < 4 { Some(Self(n)) } else { None }
    }

    const fn naga(self) -> naga::MathFunction {
        match self {
            Self(0) => naga::MathFunction::Abs,
            Self(1) => naga::MathFunction::Min,
            Self(2) => naga::MathFunction::Max,
            Self(3) => naga::MathFunction::Clamp,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
struct Args<'args>(&'args [Handle<naga::Expression>]);

impl Args<'_> {
    fn get<const N: usize>(self) -> Result<[Handle<naga::Expression>; N], BuildFnError> {
        let args: [_; N] = self.0.try_into().map_err(|_| BuildFnError)?;
        Ok(args)
    }
}

struct BuildFnError;
type FnRes = Result<Handle<naga::Expression>, BuildFnError>;
type BuildFnCall = fn(&mut Fnc<'_>, Args<'_>) -> FnRes;

fn builtins() -> Map<TypeId, BuildFnCall> {
    let mut fns: Map<TypeId, BuildFnCall> = map::make();

    fn make<V, S, const N: usize>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        V: Value,
    {
        let args: [_; N] = args.get()?.map(expr);
        Ok(fnc.do_compose::<V, S>(&args).get())
    }

    fns.insert(fnid(glam::vec2), make::<Vec2, f32, 2>);
    fns.insert(fnid(glam::vec3), make::<Vec3, f32, 3>);
    fns.insert(fnid(glam::vec4), make::<Vec4, f32, 4>);
    fns.insert(fnid(Vec2::new), make::<Vec2, f32, 2>);
    fns.insert(fnid(Vec3::new), make::<Vec3, f32, 3>);
    fns.insert(fnid(Vec4::new), make::<Vec4, f32, 4>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(glam::ivec2), make::<IVec2, i32, 2>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(IVec2::new), make::<IVec2, i32, 2>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(glam::ivec3), make::<IVec3, i32, 3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(IVec3::new), make::<IVec3, i32, 3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(glam::ivec4), make::<IVec4, i32, 4>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(IVec4::new), make::<IVec4, i32, 4>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(glam::uvec2), make::<UVec2, u32, 2>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(UVec2::new), make::<UVec2, u32, 2>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(glam::uvec3), make::<UVec3, u32, 3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(UVec3::new), make::<UVec3, u32, 3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(glam::uvec4), make::<UVec4, u32, 4>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(UVec4::new), make::<UVec4, u32, 4>);
    fns.insert(fnid(glam::mat2), make::<Mat2, Vec2, 2>);
    fns.insert(fnid(glam::mat3), make::<Mat3, Vec3, 3>);
    fns.insert(fnid(glam::mat4), make::<Mat4, Vec4, 4>);
    fns.insert(fnid(Mat2::from_cols), make::<Mat2, Vec2, 2>);
    fns.insert(fnid(Mat3::from_cols), make::<Mat3, Vec3, 3>);
    fns.insert(fnid(Mat4::from_cols), make::<Mat4, Vec4, 4>);

    fn append<V, S, R>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        R: Value,
    {
        let [vec, e] = args.get()?;
        let vec: Expr<V> = expr(vec);
        let e: Expr<S> = expr(e);
        Ok(fnc.do_compose_tuple::<R, _, _>((vec, e)).get())
    }

    fns.insert(fnid(sl::append::<Vec2>), append::<Vec2, f32, Vec3>);
    fns.insert(fnid(sl::append::<Vec3>), append::<Vec3, f32, Vec4>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::append::<IVec2>), append::<IVec2, i32, IVec3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::append::<IVec3>), append::<IVec3, i32, IVec4>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::append::<UVec2>), append::<UVec2, u32, UVec3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::append::<UVec3>), append::<UVec3, u32, UVec4>);

    fn prepend<V, S, R>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        R: Value,
    {
        let [e, vec] = args.get()?;
        let e: Expr<S> = expr(e);
        let vec: Expr<V> = expr(vec);
        Ok(fnc.do_compose_tuple::<R, _, _>((e, vec)).get())
    }

    fns.insert(fnid(sl::prepend::<Vec2>), prepend::<Vec2, f32, Vec3>);
    fns.insert(fnid(sl::prepend::<Vec3>), prepend::<Vec3, f32, Vec4>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::prepend::<IVec2>), prepend::<IVec2, i32, IVec3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::prepend::<IVec3>), prepend::<IVec3, i32, IVec4>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::prepend::<UVec2>), prepend::<UVec2, u32, UVec3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::prepend::<UVec3>), prepend::<UVec3, u32, UVec4>);

    fn concat<V, R>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        R: Value,
    {
        let [a, b] = args.get()?;
        let a: Expr<V> = expr(a);
        let b: Expr<V> = expr(b);
        Ok(fnc.do_compose_tuple::<R, _, _>((a, b)).get())
    }

    fns.insert(fnid(sl::concat::<Vec2>), concat::<Vec2, Vec4>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::concat::<IVec2>), concat::<IVec2, IVec4>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::concat::<UVec2>), concat::<UVec2, UVec4>);

    fn splat<V, S>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        V: Value,
    {
        let [x] = args.get()?.map(expr);
        Ok(fnc.do_splat::<V, S>(x).get())
    }

    fns.insert(fnid(sl::splat_vec2::<f32>), splat::<Vec2, f32>);
    fns.insert(fnid(sl::splat_vec3::<f32>), splat::<Vec3, f32>);
    fns.insert(fnid(sl::splat_vec4::<f32>), splat::<Vec4, f32>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::splat_vec2::<i32>), splat::<IVec2, i32>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::splat_vec3::<i32>), splat::<IVec3, i32>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::splat_vec4::<i32>), splat::<IVec4, i32>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::splat_vec2::<u32>), splat::<UVec2, u32>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::splat_vec3::<u32>), splat::<UVec3, u32>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::splat_vec4::<u32>), splat::<UVec4, u32>);

    fn tdim<S, const D: usize>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        u32: Dim<D>,
    {
        let [texture] = args.get()?;
        let texture = expr(texture);
        Ok(fnc
            .do_image_size::<S, <u32 as Dim<D>>::Vec, D>(texture)
            .get())
    }

    fns.insert(fnid(sl::texture_dimensions::<f32, 1>), tdim::<f32, 1>);
    fns.insert(fnid(sl::texture_dimensions::<f32, 2>), tdim::<f32, 2>);
    fns.insert(fnid(sl::texture_dimensions::<f32, 3>), tdim::<f32, 3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_dimensions::<i32, 1>), tdim::<i32, 1>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_dimensions::<i32, 2>), tdim::<i32, 2>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_dimensions::<i32, 3>), tdim::<i32, 3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_dimensions::<u32, 1>), tdim::<u32, 1>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_dimensions::<u32, 2>), tdim::<u32, 2>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_dimensions::<u32, 3>), tdim::<u32, 3>);

    fn tsam<const D: usize>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        f32: Dim<D>,
    {
        let [texture, sampler, point] = args.get()?;
        let texture = expr(texture);
        let sampler = expr(sampler);
        let point = expr(point);
        Ok(fnc
            .do_image_sample::<<f32 as Dim<D>>::Vec, D>(texture, sampler, point)
            .get())
    }

    fns.insert(fnid(sl::texture_sample::<1>), tsam::<1>);
    fns.insert(fnid(sl::texture_sample::<2>), tsam::<2>);
    fns.insert(fnid(sl::texture_sample::<3>), tsam::<3>);

    fn tload<S, const D: usize>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes
    where
        S: Scalar + Dim<D> + Dim<4>,
    {
        let [texture, point] = args.get()?;
        let texture = expr(texture);
        let point = expr(point);
        Ok(fnc
            .do_image_load::<S, <S as Dim<D>>::Vec, D>(texture, point)
            .get())
    }

    fns.insert(fnid(sl::texture_load::<f32, 1>), tload::<f32, 1>);
    fns.insert(fnid(sl::texture_load::<f32, 2>), tload::<f32, 2>);
    fns.insert(fnid(sl::texture_load::<f32, 3>), tload::<f32, 3>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_load::<i32, 1>), tload::<i32, 1>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_load::<i32, 2>), tload::<i32, 2>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::texture_load::<i32, 3>), tload::<i32, 3>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_load::<u32, 1>), tload::<u32, 1>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_load::<u32, 2>), tload::<u32, 2>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::texture_load::<u32, 3>), tload::<u32, 3>);

    fn discard(fnc: &mut Fnc<'_>, _: Args<'_>) -> FnRes {
        fnc.do_discard();
        Ok(0.expr(fnc).get())
    }

    fns.insert(fnid(sl::discard), discard);

    fn math<const F: usize>(fnc: &mut Fnc<'_>, args: Args<'_>) -> FnRes {
        let fun = const { Math::from_usize(F).expect("math function").naga() };
        Ok(fnc.do_math(fun, args.0))
    }

    fns.insert(fnid(sl::abs::<f32, 1>), math::<{ Math::ABS.get() }>);
    fns.insert(fnid(sl::abs::<f32, 2>), math::<{ Math::ABS.get() }>);
    fns.insert(fnid(sl::abs::<f32, 3>), math::<{ Math::ABS.get() }>);
    fns.insert(fnid(sl::abs::<f32, 4>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::abs::<i32, 1>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::abs::<i32, 2>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::abs::<i32, 3>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::abs::<i32, 4>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::abs::<u32, 1>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::abs::<u32, 2>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::abs::<u32, 3>), math::<{ Math::ABS.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::abs::<u32, 4>), math::<{ Math::ABS.get() }>);

    fns.insert(fnid(sl::min::<f32, 1>), math::<{ Math::MIN.get() }>);
    fns.insert(fnid(sl::min::<f32, 2>), math::<{ Math::MIN.get() }>);
    fns.insert(fnid(sl::min::<f32, 3>), math::<{ Math::MIN.get() }>);
    fns.insert(fnid(sl::min::<f32, 4>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::min::<i32, 1>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::min::<i32, 2>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::min::<i32, 3>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::min::<i32, 4>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::min::<u32, 1>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::min::<u32, 2>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::min::<u32, 3>), math::<{ Math::MIN.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::min::<u32, 4>), math::<{ Math::MIN.get() }>);

    fns.insert(fnid(sl::max::<f32, 1>), math::<{ Math::MAX.get() }>);
    fns.insert(fnid(sl::max::<f32, 2>), math::<{ Math::MAX.get() }>);
    fns.insert(fnid(sl::max::<f32, 3>), math::<{ Math::MAX.get() }>);
    fns.insert(fnid(sl::max::<f32, 4>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::max::<i32, 1>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::max::<i32, 2>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::max::<i32, 3>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::max::<i32, 4>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::max::<u32, 1>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::max::<u32, 2>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::max::<u32, 3>), math::<{ Math::MAX.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::max::<u32, 4>), math::<{ Math::MAX.get() }>);

    fns.insert(fnid(sl::clamp::<f32, 1>), math::<{ Math::CLAMP.get() }>);
    fns.insert(fnid(sl::clamp::<f32, 2>), math::<{ Math::CLAMP.get() }>);
    fns.insert(fnid(sl::clamp::<f32, 3>), math::<{ Math::CLAMP.get() }>);
    fns.insert(fnid(sl::clamp::<f32, 4>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::clamp::<i32, 1>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::clamp::<i32, 2>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::clamp::<i32, 3>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathi")]
    fns.insert(fnid(sl::clamp::<i32, 4>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::clamp::<u32, 1>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::clamp::<u32, 2>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::clamp::<u32, 3>), math::<{ Math::CLAMP.get() }>);
    #[cfg(feature = "mathu")]
    fns.insert(fnid(sl::clamp::<u32, 4>), math::<{ Math::CLAMP.get() }>);

    fns
}

pub struct Imports {
    export: Map<TypeId, BuildFnCall>,
}

impl Imports {
    pub(crate) const fn new() -> Self {
        Self {
            export: map::make(),
        }
    }

    fn get(&self, id: TypeId) -> Option<BuildFnCall> {
        static BUILTINS: LazyLock<Map<TypeId, BuildFnCall>> = LazyLock::new(builtins);

        BUILTINS.get(&id).or_else(|| self.export.get(&id)).copied()
    }
}

#[derive(Clone, Copy)]
struct GlobalVariableEntry {
    var: Handle<naga::GlobalVariable>,
}

enum MakeGroup {
    Cached(u32),
    New(TypeId),
}

pub struct Irc {
    imports: Imports,
    group: u32,
    group_globals: Map<TypeId, u32>,
    location: Option<u32>,
    types: UniqueArena<naga::Type>,
    named_types: Map<TypeId, Handle<naga::Type>>,
    global_variables: Arena<naga::GlobalVariable>,
    global_map: Map<u32, Box<[GlobalVariableEntry]>>,
    global_count: u32,
    functions: Arena<naga::Function>,
    entries: Vec<naga::EntryPoint>,
    error: Option<IrcError>,
}

impl Irc {
    pub fn new(imports: Imports) -> Self {
        Self {
            imports,
            group: 0,
            group_globals: map::make(),
            location: None,
            types: UniqueArena::new(),
            named_types: map::make(),
            global_variables: Arena::new(),
            global_map: map::make(),
            global_count: 0,
            functions: Arena::new(),
            entries: vec![],
            error: None,
        }
    }

    pub fn boot<F>(&mut self, boot: Boot, f: F)
    where
        F: FnOnce(&mut Hook<'_>),
    {
        self.location = Some(0);
        f(&mut Hook::new(self, boot));
        self.location = None;
    }

    fn location(&mut self) -> Option<&mut u32> {
        self.location.as_mut()
    }

    fn add_type(&mut self, ty: Type) -> Handle<naga::Type> {
        match ty.naga() {
            Ok(inner) => {
                let ty = naga::Type { name: None, inner };
                self.types.insert(ty, Span::UNDEFINED)
            }
            Err(DynamicType { make, name }) => {
                if let Some(&handle) = self.named_types.get(&name) {
                    return handle;
                }

                let ty = naga::Type {
                    name: None,
                    inner: make(TypeBuilder { irc: self }),
                };

                let handle = self.types.insert(ty, Span::UNDEFINED);
                self.named_types.insert(name, handle);
                handle
            }
        }
    }

    pub(crate) fn new_group(&mut self) {
        self.group += 1;
    }

    pub(crate) fn add_uniform<V, D>(&mut self, binding: u32) -> GlobalVariable<Uniform<V, D>>
    where
        V: Value,
    {
        let ty = self.add_type(V::NAGA);
        let var = self.add_global(ty, naga::AddressSpace::Uniform, binding);
        GlobalVariable {
            var,
            ty: PhantomData,
        }
    }

    pub(crate) fn add_storage<V, D>(&mut self, binding: u32) -> GlobalVariable<Storage<V, D>>
    where
        V: MaybeSizedValue + ?Sized,
    {
        let ty = self.add_type(V::NAGA);
        let var = self.add_global(
            ty,
            naga::AddressSpace::Storage {
                access: naga::StorageAccess::LOAD,
            },
            binding,
        );

        GlobalVariable {
            var,
            ty: PhantomData,
        }
    }

    fn add_global(
        &mut self,
        ty: Handle<naga::Type>,
        space: naga::AddressSpace,
        binding: u32,
    ) -> Handle<naga::GlobalVariable> {
        if let naga::AddressSpace::Uniform = space
            && let Ok(naga::Type {
                inner: naga::TypeInner::Array { stride, .. },
                ..
            }) = self.types.get_handle(ty)
            && !stride.is_multiple_of(16)
        {
            self.error = Some(IrcError::ArrayStride {
                actual: *stride,
                required: 16,
            });
        }

        if let naga::AddressSpace::Storage { .. } = space
            && let Ok(naga::Type {
                inner: naga::TypeInner::Struct { span, .. },
                ..
            }) = self.types.get_handle(ty)
            && !span.is_multiple_of(8)
        {
            self.error = Some(IrcError::StructSpan {
                actual: *span,
                required: 8,
            });
        }

        let global = naga::GlobalVariable {
            name: None,
            space,
            binding: Some(naga::ResourceBinding {
                group: self.group,
                binding,
            }),
            ty,
            init: None,
        };

        self.global_variables.append(global, Span::UNDEFINED)
    }

    fn add_global_descriptor<T>(&mut self, binding: u32) -> GlobalVariable<T>
    where
        T: Descriptor,
    {
        let ty = self.add_type(T::NAGA);
        let global = naga::GlobalVariable {
            name: None,
            space: naga::AddressSpace::Handle,
            binding: Some(naga::ResourceBinding {
                group: self.group,
                binding,
            }),
            ty,
            init: None,
        };

        let var = self.global_variables.append(global, Span::UNDEFINED);
        GlobalVariable {
            var,
            ty: PhantomData,
        }
    }

    fn add_global_map(&mut self, vars: Box<[GlobalVariableEntry]>) -> u32 {
        let id = self.global_count;
        self.global_count += 1;
        self.global_map.insert(id, vars);
        id
    }

    pub fn build(self) -> Comp<naga::Module> {
        if let Some(e) = self.error {
            return Err(e.into());
        }

        Ok(naga::Module {
            types: self.types,
            global_variables: self.global_variables,
            functions: self.functions,
            entry_points: self.entries,
            ..naga::Module::default()
        })
    }
}

#[derive(Debug)]
enum IrcError {
    ArrayStride { actual: u32, required: u32 },
    StructSpan { actual: u32, required: u32 },
}

pub type Comp<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrorRepr>);

impl Error {
    fn new(inner: ErrorInner) -> Self {
        Self(Box::new(ErrorRepr {
            inner,
            context: String::new(),
        }))
    }

    pub fn add_context<D>(mut self, d: D) -> Self
    where
        D: fmt::Display,
    {
        _ = write!(self.0.context, "{d}");
        self
    }
}

impl From<IrcError> for Error {
    fn from(e: IrcError) -> Self {
        Self::new(ErrorInner::Irc(e))
    }
}

impl From<FncError> for Error {
    fn from(e: FncError) -> Self {
        Self::new(ErrorInner::Fnc(e))
    }
}

impl From<ConstructorError> for Error {
    fn from(e: ConstructorError) -> Self {
        Self::new(ErrorInner::Constructor(e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.inner {
            ErrorInner::Irc(IrcError::ArrayStride { actual, required }) => write!(
                f,
                "the array stride {actual} is not a multiple of the required alignment {required}",
            )?,
            ErrorInner::Irc(IrcError::StructSpan { actual, required }) => write!(
                f,
                "the struct span {actual}, is not a multiple of the required alignment {required}",
            )?,
            ErrorInner::Fnc(FncError::CallImport) => f.write_str("failed to call import")?,
            ErrorInner::Fnc(FncError::CallFailed) => f.write_str("import call failed")?,
            ErrorInner::Constructor(ConstructorError) => {
                f.write_str("failed to construct struct")?;
            }
        }

        if !self.0.context.is_empty() {
            f.write_char('\n')?;
            f.write_str(&self.0.context)?;
        }

        Ok(())
    }
}

impl error::Error for Error {}

#[derive(Debug)]
struct ErrorRepr {
    inner: ErrorInner,
    context: String,
}

#[derive(Debug)]
enum ErrorInner {
    Irc(IrcError),
    Fnc(FncError),
    Constructor(ConstructorError),
}

pub enum InputKind {
    Value,
    Group,
}

pub trait Input {
    const KIND: InputKind;
    type Ref: ?Sized;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<Self::Ref> + use<Self>;

    fn init(irc: &mut Irc) {
        _ = irc;
    }
}

impl<V> Input for V
where
    V: Value,
{
    const KIND: InputKind = InputKind::Value;
    type Ref = Self;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<Self::Ref> + use<V> {
        fnc.add_argument()
    }

    fn init(irc: &mut Irc) {
        irc.add_type(V::NAGA);
    }
}

pub trait Output {
    fn output(fnc: &mut Fnc<'_>, stage: Stage);
}

impl<V> Output for V
where
    V: Value,
{
    fn output(fnc: &mut Fnc<'_>, stage: Stage) {
        fnc.add_result_type::<V>(stage);
    }
}

pub trait Descriptor {
    const NAGA: Type;
    const FORMAT: GroupFormat;
}

pub(crate) const fn dimension(d: usize) -> naga::ImageDimension {
    match d {
        1 => naga::ImageDimension::D1,
        2 => naga::ImageDimension::D2,
        3 => naga::ImageDimension::D3,
        _ => panic!("unsupported dimension"),
    }
}

pub trait Dim<const D: usize> {
    type Vec;
    fn splat(self) -> Self::Vec;
}

impl Dim<1> for f32 {
    type Vec = Self;

    fn splat(self) -> Self::Vec {
        self
    }
}

impl Dim<2> for f32 {
    type Vec = Vec2;

    fn splat(self) -> Self::Vec {
        Vec2::splat(self)
    }
}

impl Dim<3> for f32 {
    type Vec = Vec3;

    fn splat(self) -> Self::Vec {
        Vec3::splat(self)
    }
}

impl Dim<4> for f32 {
    type Vec = Vec4;

    fn splat(self) -> Self::Vec {
        Vec4::splat(self)
    }
}

impl Dim<1> for i32 {
    type Vec = Self;

    fn splat(self) -> Self::Vec {
        self
    }
}

impl Dim<2> for i32 {
    type Vec = IVec2;

    fn splat(self) -> Self::Vec {
        IVec2::splat(self)
    }
}

impl Dim<3> for i32 {
    type Vec = IVec3;

    fn splat(self) -> Self::Vec {
        IVec3::splat(self)
    }
}

impl Dim<4> for i32 {
    type Vec = IVec4;

    fn splat(self) -> Self::Vec {
        IVec4::splat(self)
    }
}

impl Dim<1> for u32 {
    type Vec = Self;

    fn splat(self) -> Self::Vec {
        self
    }
}

impl Dim<2> for u32 {
    type Vec = UVec2;

    fn splat(self) -> Self::Vec {
        UVec2::splat(self)
    }
}

impl Dim<3> for u32 {
    type Vec = UVec3;

    fn splat(self) -> Self::Vec {
        UVec3::splat(self)
    }
}

impl Dim<4> for u32 {
    type Vec = UVec4;

    fn splat(self) -> Self::Vec {
        UVec4::splat(self)
    }
}

pub trait ExprTuple<const N: usize>: Sized {
    type Tuple;
    fn get(self) -> [Handle<naga::Expression>; N];
}

impl ExprTuple<0> for () {
    type Tuple = ();

    fn get(self) -> [Handle<naga::Expression>; 0] {
        []
    }
}

impl<A> ExprTuple<1> for (Expr<A>,) {
    type Tuple = (A,);

    fn get(self) -> [Handle<naga::Expression>; 1] {
        [self.0.get()]
    }
}

impl<A, B> ExprTuple<2> for (Expr<A>, Expr<B>) {
    type Tuple = (A, B);

    fn get(self) -> [Handle<naga::Expression>; 2] {
        [self.0.get(), self.1.get()]
    }
}

impl<A, B, C> ExprTuple<3> for (Expr<A>, Expr<B>, Expr<C>) {
    type Tuple = (A, B, C);

    fn get(self) -> [Handle<naga::Expression>; 3] {
        [self.0.get(), self.1.get(), self.2.get()]
    }
}

impl<A, B, C, D> ExprTuple<4> for (Expr<A>, Expr<B>, Expr<C>, Expr<D>) {
    type Tuple = (A, B, C, D);

    fn get(self) -> [Handle<naga::Expression>; 4] {
        [self.0.get(), self.1.get(), self.2.get(), self.3.get()]
    }
}

impl<A, B, C, D, E> ExprTuple<5> for (Expr<A>, Expr<B>, Expr<C>, Expr<D>, Expr<E>) {
    type Tuple = (A, B, C, D, E);

    fn get(self) -> [Handle<naga::Expression>; 5] {
        [
            self.0.get(),
            self.1.get(),
            self.2.get(),
            self.3.get(),
            self.4.get(),
        ]
    }
}

fn expr<T>(handle: Handle<naga::Expression>) -> Expr<T>
where
    T: ?Sized,
{
    Expr(handle, PhantomData)
}

pub struct Expr<T>(Handle<naga::Expression>, PhantomData<T>)
where
    T: ?Sized;

impl<T> Expr<T>
where
    T: ?Sized,
{
    const fn get(self) -> Handle<naga::Expression> {
        self.0
    }
}

impl<T> Clone for Expr<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Expr<T> where T: ?Sized {}

pub struct Argument<T>(u32, PhantomData<T>)
where
    T: ?Sized;

impl<T> Clone for Argument<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Argument<T> {}

impl<T> Reference<T> for Argument<T> {
    fn reference(self) -> TakeReference<T> {
        TakeReference::Argument(self)
    }
}

pub struct GlobalVariable<T>
where
    T: ?Sized,
{
    var: Handle<naga::GlobalVariable>,
    ty: PhantomData<T>,
}

impl<T> GlobalVariable<T>
where
    T: ?Sized,
{
    fn entry(self) -> GlobalVariableEntry {
        GlobalVariableEntry { var: self.var }
    }
}

impl<T> Clone for GlobalVariable<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GlobalVariable<T> where T: ?Sized {}

impl<T> Reference<T> for GlobalVariable<T>
where
    T: ?Sized,
{
    fn reference(self) -> TakeReference<T> {
        TakeReference::GlobalVariable(self)
    }
}

pub struct GlobalVariables<T>(u32, PhantomData<T>)
where
    T: ?Sized;

impl<T> GlobalVariables<T>
where
    T: ?Sized,
{
    fn new(n: u32) -> Self {
        Self(n, PhantomData)
    }
}

impl<T> Clone for GlobalVariables<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GlobalVariables<T> where T: ?Sized {}

impl<T> Reference<T> for GlobalVariables<T>
where
    T: ?Sized,
{
    fn reference(self) -> TakeReference<T> {
        TakeReference::GlobalVariables(self.0)
    }
}

pub struct Variable<T>(Handle<naga::LocalVariable>, PhantomData<T>)
where
    T: ?Sized;

impl<T> Clone for Variable<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Variable<T> {}

impl<T> Reference<T> for Variable<T> {
    fn reference(self) -> TakeReference<T> {
        TakeReference::Variable(self)
    }
}

pub trait Reference<T>: Copy
where
    T: ?Sized,
{
    fn reference(self) -> TakeReference<T>;
}

pub enum TakeReference<T>
where
    T: ?Sized,
{
    Argument(Argument<T>),
    Variable(Variable<T>),
    GlobalVariable(GlobalVariable<T>),
    GlobalVariables(u32),
}

pub enum Pointer<T>
where
    T: ?Sized,
{
    Load(Expr<T>),
    Noop(Expr<T>),
    GlobalVariables(u32),
}

impl<T> fmt::Debug for Pointer<T>
where
    T: ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(_) => f.debug_tuple("Load").field(&"..").finish(),
            Self::Noop(_) => f.debug_tuple("Noop").field(&"..").finish(),
            Self::GlobalVariables(arg0) => f.debug_tuple("GlobalVariables").field(arg0).finish(),
        }
    }
}

impl<T> Clone for Pointer<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Pointer<T> where T: ?Sized {}

pub struct Op<T>(naga::Expression, PhantomData<T>);

const fn binop<L, R, O>(l: Expr<L>, r: Expr<R>, naga: naga::BinaryOperator) -> Op<O> {
    Op(
        naga::Expression::Binary {
            op: naga,
            left: l.get(),
            right: r.get(),
        },
        PhantomData,
    )
}

pub fn add<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Add<R>,
{
    binop(l, r, naga::BinaryOperator::Add)
}

pub fn sub<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Sub<R>,
{
    binop(l, r, naga::BinaryOperator::Subtract)
}

pub fn mul<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Mul<R>,
{
    binop(l, r, naga::BinaryOperator::Multiply)
}

pub fn div<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Div<R>,
{
    binop(l, r, naga::BinaryOperator::Divide)
}

pub fn rem<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Rem<R>,
{
    binop(l, r, naga::BinaryOperator::Modulo)
}

pub fn shl<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Shl<R>,
{
    binop(l, r, naga::BinaryOperator::ShiftLeft)
}

pub fn shr<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::Shr<R>,
{
    binop(l, r, naga::BinaryOperator::ShiftRight)
}

pub fn binand<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::BitAnd<R>,
{
    binop(l, r, naga::BinaryOperator::And)
}

pub fn binor<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::BitOr<R>,
{
    binop(l, r, naga::BinaryOperator::InclusiveOr)
}

pub fn binxor<L, R>(l: Expr<L>, r: Expr<R>) -> Op<L::Output>
where
    L: ops::BitXor<R>,
{
    binop(l, r, naga::BinaryOperator::ExclusiveOr)
}

pub fn and(l: Expr<bool>, r: Expr<bool>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::LogicalAnd)
}

pub fn or(l: Expr<bool>, r: Expr<bool>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::LogicalOr)
}

pub fn eq<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::Equal)
}

pub fn ne<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::NotEqual)
}

pub fn lt<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::Less)
}

pub fn le<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::LessEqual)
}

pub fn gt<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::Greater)
}

pub fn ge<T>(l: Expr<T>, r: Expr<T>) -> Op<bool> {
    binop(l, r, naga::BinaryOperator::GreaterEqual)
}

#[diagnostic::on_unimplemented(
    message = "type `{Self}` is not a scalar in the shader",
    label = "not a shader scalar"
)]
pub trait Scalar: Sized + Copy + 'static {
    const NAGA: naga::Scalar;
    const LITERAL: fn(Self) -> naga::Literal;
}

impl Scalar for bool {
    const NAGA: naga::Scalar = naga::Scalar::BOOL;
    const LITERAL: fn(Self) -> naga::Literal = naga::Literal::Bool;
}

impl Scalar for f32 {
    const NAGA: naga::Scalar = naga::Scalar::F32;
    const LITERAL: fn(Self) -> naga::Literal = naga::Literal::F32;
}

impl Scalar for i32 {
    const NAGA: naga::Scalar = naga::Scalar::I32;
    const LITERAL: fn(Self) -> naga::Literal = naga::Literal::I32;
}

impl Scalar for u32 {
    const NAGA: naga::Scalar = naga::Scalar::U32;
    const LITERAL: fn(Self) -> naga::Literal = naga::Literal::U32;
}

pub trait GroupMember {
    const FORMAT: GroupFormat;
    type Global: ?Sized;
    fn global(irc: &mut Irc, binding: u32) -> GlobalVariable<Self::Global>;
}

impl<D> GroupMember for D
where
    D: Descriptor,
{
    const FORMAT: GroupFormat = D::FORMAT;
    type Global = Self;

    fn global(irc: &mut Irc, binding: u32) -> GlobalVariable<Self::Global> {
        irc.add_global_descriptor::<Self>(binding)
    }
}

pub struct GroupBuilder<'irc, G> {
    irc: &'irc mut Irc,
    make: MakeGroup,
    binding: u32,
    vars: Vec<GlobalVariableEntry>,
    ty: PhantomData<G>,
}

impl<G> GroupBuilder<'_, G> {
    pub fn add_member<T>(mut self) -> Self
    where
        T: GroupMember,
    {
        if let MakeGroup::Cached(_) = self.make {
            return self;
        }

        let var = T::global(self.irc, self.binding);
        self.binding += 1;
        self.vars.push(var.entry());
        self
    }

    pub fn build(self) -> impl Reference<G> + use<G> {
        match self.make {
            MakeGroup::Cached(n) => GlobalVariables::new(n),
            MakeGroup::New(id) => {
                let n = self.irc.add_global_map(self.vars.into_boxed_slice());
                self.irc.group_globals.insert(id, n);
                let global = GlobalVariables::new(n);
                self.irc.new_group();
                global
            }
        }
    }
}

pub struct TypeBuilder<'irc> {
    irc: &'irc mut Irc,
}

impl<'irc> TypeBuilder<'irc> {
    pub fn build_struct<S>(self) -> StructBuilder<'irc> {
        StructBuilder {
            irc: self.irc,
            span: size_of::<S>() as u32,
            members: vec![],
        }
    }

    fn build_array<V, const N: usize>(self) -> naga::TypeInner
    where
        V: Value,
    {
        let size = const {
            assert!(N <= u32::MAX as usize, "too large array");
            NonZero::new(N as u32).expect("array size cannot be zero")
        };

        let base = self.irc.add_type(V::NAGA);
        naga::TypeInner::Array {
            base,
            size: naga::ArraySize::Constant(size),
            stride: size_of::<V>() as u32,
        }
    }

    pub(crate) fn build_dynamic_array<V>(self) -> naga::TypeInner
    where
        V: Value,
    {
        let base = self.irc.add_type(V::NAGA);
        naga::TypeInner::Array {
            base,
            size: naga::ArraySize::Dynamic,
            stride: size_of::<V>() as u32,
        }
    }
}

pub enum Binding {
    None,
    Location(u32),
    Position,
    Index,
}

impl Binding {
    fn naga<V>(self) -> Option<naga::Binding>
    where
        V: Value,
    {
        match self {
            Self::None => None,
            Self::Location(location) => {
                let mut bind = naga::Binding::Location {
                    location,
                    interpolation: None,
                    sampling: None,
                    blend_src: None,
                    per_primitive: false,
                };

                if let Ok(inner) = const { V::NAGA.naga() } {
                    bind.apply_default_interpolation(&inner);
                }

                Some(bind)
            }
            Self::Position => Some(naga::Binding::BuiltIn(naga::BuiltIn::Position {
                invariant: false,
            })),
            Self::Index => Some(naga::Binding::BuiltIn(naga::BuiltIn::VertexIndex)),
        }
    }
}

pub struct StructBuilder<'irc> {
    irc: &'irc mut Irc,
    span: u32,
    members: Vec<naga::StructMember>,
}

impl StructBuilder<'_> {
    pub fn add_member<V>(mut self, name: &str, mut binding: Binding, offset: u32) -> Self
    where
        V: Value,
    {
        if let Some(location) = self.irc.location()
            && let binding @ Binding::None = &mut binding
        {
            *binding = Binding::Location(*location);
            *location += 1;
        }

        let member = naga::StructMember {
            name: Some(name.to_owned()),
            ty: self.irc.add_type(V::NAGA),
            binding: binding.naga::<V>(),
            offset,
        };

        self.members.push(member);
        self
    }

    pub fn build(mut self) -> naga::TypeInner {
        self.members.sort_unstable_by_key(|m| m.offset);
        naga::TypeInner::Struct {
            members: self.members,
            span: self.span,
        }
    }
}

const fn vecsize(size: usize) -> naga::VectorSize {
    match size {
        2 => naga::VectorSize::Bi,
        3 => naga::VectorSize::Tri,
        4 => naga::VectorSize::Quad,
        _ => panic!("non vector size"),
    }
}

#[derive(Clone, Copy)]
pub struct DynamicType {
    make: fn(TypeBuilder<'_>) -> naga::TypeInner,
    name: TypeId,
}

#[derive(Clone, Copy)]
pub enum Type {
    Scalar(naga::Scalar),
    Vector {
        size: naga::VectorSize,
        scalar: naga::Scalar,
    },
    Matrix {
        columns: naga::VectorSize,
        rows: naga::VectorSize,
        scalar: naga::Scalar,
    },
    Image {
        dim: naga::ImageDimension,
        arrayed: bool,
        class: naga::ImageClass,
    },
    Sampler {
        comparison: bool,
    },
    Dynamic(DynamicType),
}

impl Type {
    const fn vec<S>(size: usize) -> Self
    where
        S: Scalar,
    {
        Self::Vector {
            size: vecsize(size),
            scalar: S::NAGA,
        }
    }

    const fn mat<S>(size: usize) -> Self
    where
        S: Scalar,
    {
        Self::Matrix {
            columns: vecsize(size),
            rows: vecsize(size),
            scalar: S::NAGA,
        }
    }

    pub const fn dynamic<T>(make: fn(TypeBuilder<'_>) -> naga::TypeInner) -> Self
    where
        T: ?Sized + 'static,
    {
        Self::Dynamic(DynamicType {
            make,
            name: TypeId::of::<T>(),
        })
    }

    const fn naga(self) -> Result<naga::TypeInner, DynamicType> {
        match self {
            Self::Scalar(scalar) => Ok(naga::TypeInner::Scalar(scalar)),
            Self::Vector { size, scalar } => Ok(naga::TypeInner::Vector { size, scalar }),
            Self::Matrix {
                columns,
                rows,
                scalar,
            } => Ok(naga::TypeInner::Matrix {
                columns,
                rows,
                scalar,
            }),
            Self::Image {
                dim,
                arrayed,
                class,
            } => Ok(naga::TypeInner::Image {
                dim,
                arrayed,
                class,
            }),
            Self::Sampler { comparison } => Ok(naga::TypeInner::Sampler { comparison }),
            Self::Dynamic(dynty) => Err(dynty),
        }
    }

    fn eq_types(self, rhs: Self) -> bool {
        match (self.naga(), rhs.naga()) {
            (Ok(me), Ok(rhs)) => me == rhs,
            (Err(DynamicType { name: me, .. }), Err(DynamicType { name: rhs, .. })) => me == rhs,
            _ => false,
        }
    }
}

#[diagnostic::on_unimplemented(
    message = "type `{Self}` is not a value in the shader",
    label = "not a shader value"
)]
pub trait Value: Sized + Copy + 'static {
    const NAGA: Type;
    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self>;
}

impl<S> Value for S
where
    S: Scalar,
{
    const NAGA: Type = Type::Scalar(S::NAGA);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        fnc.do_literal(self)
    }
}

impl<V, const N: usize> Value for [V; N]
where
    V: Value,
{
    const NAGA: Type = Type::dynamic::<Self>(|b| b.build_array::<V, N>());

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let items = self.map(|v| v.expr(fnc));
        fnc.do_compose_array(items)
    }
}

pub trait MaybeSizedValue {
    const NAGA: Type;
}

impl<V> MaybeSizedValue for V
where
    V: Value,
{
    const NAGA: Type = V::NAGA;
}

impl<V> MaybeSizedValue for [V]
where
    V: Value,
{
    const NAGA: Type = Type::dynamic::<Self>(|b| b.build_dynamic_array::<V>());
}

pub struct ArrayMethods<V, const N: usize> {
    pub len: Method<[V; N], u32>,
}

impl<V, const N: usize> Methods for [V; N] {
    type Methods = ArrayMethods<V, N>;

    const METHODS: Self::Methods = ArrayMethods {
        len: {
            assert!(N <= u32::MAX as usize, "too large array");
            method(|fnc, _| (N as u32).expr(fnc))
        },
    };
}

impl Value for Vec2 {
    const NAGA: Type = Type::vec::<f32>(2);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            fnc.do_compose(&[x, y])
        }
    }
}

impl Value for Vec3 {
    const NAGA: Type = Type::vec::<f32>(3);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            fnc.do_compose(&[x, y, z])
        }
    }
}

impl Value for Vec4 {
    const NAGA: Type = Type::vec::<f32>(4);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            let w = self.w.expr(fnc);
            fnc.do_compose(&[x, y, z, w])
        }
    }
}

impl Value for IVec2 {
    const NAGA: Type = Type::vec::<i32>(2);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            fnc.do_compose(&[x, y])
        }
    }
}

impl Value for IVec3 {
    const NAGA: Type = Type::vec::<i32>(3);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            fnc.do_compose(&[x, y, z])
        }
    }
}

impl Value for IVec4 {
    const NAGA: Type = Type::vec::<i32>(4);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            let w = self.w.expr(fnc);
            fnc.do_compose(&[x, y, z, w])
        }
    }
}

impl Value for UVec2 {
    const NAGA: Type = Type::vec::<u32>(2);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            fnc.do_compose(&[x, y])
        }
    }
}

impl Value for UVec3 {
    const NAGA: Type = Type::vec::<u32>(3);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            fnc.do_compose(&[x, y, z])
        }
    }
}

impl Value for UVec4 {
    const NAGA: Type = Type::vec::<u32>(4);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let x = self.x.expr(fnc);
        if self == Self::splat(self.x) {
            fnc.do_splat(x)
        } else {
            let y = self.y.expr(fnc);
            let z = self.z.expr(fnc);
            let w = self.w.expr(fnc);
            fnc.do_compose(&[x, y, z, w])
        }
    }
}

impl Value for Mat2 {
    const NAGA: Type = Type::mat::<f32>(2);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let [x, y] = self.to_cols_array_2d().map(Vec2::from);
        let x = x.expr(fnc);
        let y = y.expr(fnc);
        fnc.do_compose(&[x, y])
    }
}

impl Value for Mat3 {
    const NAGA: Type = Type::mat::<f32>(3);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let [x, y, z] = self.to_cols_array_2d().map(Vec3::from);
        let x = x.expr(fnc);
        let y = y.expr(fnc);
        let z = z.expr(fnc);
        fnc.do_compose(&[x, y, z])
    }
}

impl Value for Mat4 {
    const NAGA: Type = Type::mat::<f32>(4);

    fn expr(self, fnc: &mut Fnc<'_>) -> Expr<Self> {
        let [x, y, z, w] = self.to_cols_array_2d().map(Vec4::from);
        let x = x.expr(fnc);
        let y = y.expr(fnc);
        let z = z.expr(fnc);
        let w = w.expr(fnc);
        fnc.do_compose(&[x, y, z, w])
    }
}

pub trait Composite {
    type Output;
}

impl<T, const N: usize> Composite for [T; N] {
    type Output = T;
}

impl<T> Composite for [T] {
    type Output = T;
}

impl Composite for Vec2 {
    type Output = f32;
}

impl Composite for Vec3 {
    type Output = f32;
}

impl Composite for Vec4 {
    type Output = f32;
}

impl Composite for IVec2 {
    type Output = i32;
}

impl Composite for IVec3 {
    type Output = i32;
}

impl Composite for IVec4 {
    type Output = i32;
}

impl Composite for UVec2 {
    type Output = u32;
}

impl Composite for UVec3 {
    type Output = u32;
}

impl Composite for UVec4 {
    type Output = u32;
}

impl Composite for Mat2 {
    type Output = Vec2;
}

impl Composite for Mat3 {
    type Output = Vec3;
}

impl Composite for Mat4 {
    type Output = Vec4;
}

pub trait BaseAccess {
    type Base: ?Sized;
    type Output<O>;
    fn base_access<O>(self, access: Access<Self::Base, O>, fnc: &mut Fnc<'_>) -> Self::Output<O>;
    fn base_index<O, I>(self, index: Expr<I>, fnc: &mut Fnc<'_>) -> Self::Output<O>;
}

impl<T> BaseAccess for Expr<T>
where
    T: ?Sized,
{
    type Base = T;
    type Output<O> = Expr<O>;

    fn base_access<O>(self, access: Access<Self::Base, O>, fnc: &mut Fnc<'_>) -> Self::Output<O> {
        fnc.do_access_expr(self, access)
    }

    fn base_index<O, I>(self, index: Expr<I>, fnc: &mut Fnc<'_>) -> Self::Output<O> {
        fnc.do_index_expr(self, index)
    }
}

impl<T> BaseAccess for Pointer<T>
where
    T: ?Sized,
{
    type Base = T;
    type Output<O> = Pointer<O>;

    fn base_access<O>(self, access: Access<Self::Base, O>, fnc: &mut Fnc<'_>) -> Self::Output<O> {
        fnc.do_access_pointer(self, access)
    }

    fn base_index<O, I>(self, index: Expr<I>, fnc: &mut Fnc<'_>) -> Self::Output<O> {
        fnc.do_load_by_index(self, index)
    }
}

pub struct Access<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    index: u32,
    base: PhantomData<B>,
    output: PhantomData<O>,
}

impl<B, O> Access<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    fn index(self) -> u32 {
        self.index
    }
}

pub const fn index<B, O>(index: u32) -> Access<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    Access {
        index,
        base: PhantomData,
        output: PhantomData,
    }
}

macro_rules! sw {
    (x) => {
        naga::SwizzleComponent::X
    };

    (y) => {
        naga::SwizzleComponent::Y
    };

    (z) => {
        naga::SwizzleComponent::Z
    };

    (w) => {
        naga::SwizzleComponent::W
    };

    ($a:ident $b:ident) => {
        Swizzle::N2([sw!($a), sw!($b)], PhantomData, PhantomData)
    };

    ($a:ident $b:ident $c:ident) => {
        Swizzle::N3([sw!($a), sw!($b), sw!($c)], PhantomData, PhantomData)
    };

    ($a:ident $b:ident $c:ident $d:ident) => {
        Swizzle::N4(
            [sw!($a), sw!($b), sw!($c), sw!($d)],
            PhantomData,
            PhantomData,
        )
    };
}

pub enum Swizzle<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    N2([naga::SwizzleComponent; 2], PhantomData<B>, PhantomData<O>),
    N3([naga::SwizzleComponent; 3], PhantomData<B>, PhantomData<O>),
    N4([naga::SwizzleComponent; 4], PhantomData<B>, PhantomData<O>),
}

impl<B, O> Swizzle<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    fn expr(self, vector: Handle<naga::Expression>) -> naga::Expression {
        naga::Expression::Swizzle {
            size: match self {
                Self::N2(..) => naga::VectorSize::Bi,
                Self::N3(..) => naga::VectorSize::Tri,
                Self::N4(..) => naga::VectorSize::Quad,
            },
            vector,
            pattern: match self {
                Self::N2([x, y], ..) => {
                    [x, y, naga::SwizzleComponent::X, naga::SwizzleComponent::X]
                }
                Self::N3([x, y, z], ..) => [x, y, z, naga::SwizzleComponent::X],
                Self::N4(n, ..) => n,
            },
        }
    }
}

impl<B, O> Clone for Swizzle<B, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B, O> Copy for Swizzle<B, O> {}

pub struct Constructor<T> {
    components: Vec<Option<Handle<naga::Expression>>>,
    base: PhantomData<T>,
}

impl<T> Constructor<T> {
    pub fn set_field<M, F>(&mut self, expr: Expr<M>, f: F)
    where
        T: Fields,
        F: FnOnce(<T as Fields>::Fields) -> Access<T, M>,
    {
        let index = f(T::FIELDS).index() as usize;
        if index >= self.components.len() {
            self.components.resize(index + 1, None);
        }

        self.components[index] = Some(expr.get());
    }

    pub fn build(self, fnc: &mut Fnc<'_>) -> Comp<Expr<T>>
    where
        T: Value,
    {
        let ty = fnc.irc.add_type(T::NAGA);
        let components = self
            .components
            .into_iter()
            .collect::<Option<_>>()
            .ok_or(ConstructorError)?;

        let ex = naga::Expression::Compose { ty, components };
        Ok(expr(fnc.add_expr(ex)))
    }
}

impl<T> Default for Constructor<T> {
    fn default() -> Self {
        Self {
            components: vec![],
            base: PhantomData,
        }
    }
}

#[derive(Debug)]
struct ConstructorError;

pub trait Fields {
    type Tuple;
    type Fields;
    const FIELDS: Self::Fields;
}

pub struct Vec2Fields<V, S> {
    pub x: Access<V, S>,
    pub y: Access<V, S>,
}

impl Fields for Vec2 {
    type Tuple = (f32, f32);
    type Fields = Vec2Fields<Self, f32>;

    const FIELDS: Vec2Fields<Self, f32> = Vec2Fields {
        x: index(0),
        y: index(1),
    };
}

impl Fields for IVec2 {
    type Tuple = (i32, i32);
    type Fields = Vec2Fields<Self, i32>;

    const FIELDS: Vec2Fields<Self, i32> = Vec2Fields {
        x: index(0),
        y: index(1),
    };
}

impl Fields for UVec2 {
    type Tuple = (u32, u32);
    type Fields = Vec2Fields<Self, u32>;

    const FIELDS: Vec2Fields<Self, u32> = Vec2Fields {
        x: index(0),
        y: index(1),
    };
}

pub struct Vec3Fields<V, S> {
    pub x: Access<V, S>,
    pub y: Access<V, S>,
    pub z: Access<V, S>,
}

impl Fields for Vec3 {
    type Tuple = (f32, f32, f32);
    type Fields = Vec3Fields<Self, f32>;

    const FIELDS: Vec3Fields<Self, f32> = Vec3Fields {
        x: index(0),
        y: index(1),
        z: index(2),
    };
}

impl Fields for IVec3 {
    type Tuple = (i32, i32, i32);
    type Fields = Vec3Fields<Self, i32>;

    const FIELDS: Vec3Fields<Self, i32> = Vec3Fields {
        x: index(0),
        y: index(1),
        z: index(2),
    };
}

impl Fields for UVec3 {
    type Tuple = (u32, u32, u32);
    type Fields = Vec3Fields<Self, u32>;

    const FIELDS: Vec3Fields<Self, u32> = Vec3Fields {
        x: index(0),
        y: index(1),
        z: index(2),
    };
}

pub struct Vec4Fields<V, S> {
    pub x: Access<V, S>,
    pub y: Access<V, S>,
    pub z: Access<V, S>,
    pub w: Access<V, S>,
}

impl Fields for Vec4 {
    type Tuple = (f32, f32, f32, f32);
    type Fields = Vec4Fields<Self, f32>;

    const FIELDS: Vec4Fields<Self, f32> = Vec4Fields {
        x: index(0),
        y: index(1),
        z: index(2),
        w: index(3),
    };
}

impl Fields for IVec4 {
    type Tuple = (i32, i32, i32, i32);
    type Fields = Vec4Fields<Self, i32>;

    const FIELDS: Vec4Fields<Self, i32> = Vec4Fields {
        x: index(0),
        y: index(1),
        z: index(2),
        w: index(3),
    };
}

impl Fields for UVec4 {
    type Tuple = (u32, u32, u32, u32);
    type Fields = Vec4Fields<Self, u32>;

    const FIELDS: Vec4Fields<Self, u32> = Vec4Fields {
        x: index(0),
        y: index(1),
        z: index(2),
        w: index(3),
    };
}

pub trait Methods {
    type Methods;
    const METHODS: Self::Methods;
}

pub enum Method<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    Swizzle(Swizzle<B, O>),
    Expr(fn(&mut Fnc<'_>, Expr<B>) -> Expr<O>),
    Load,
    Noop,
}

const fn swizzle<B, O>(swizzle: Swizzle<B, O>) -> Method<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    Method::Swizzle(swizzle)
}

pub(crate) const fn method<B, O>(e: fn(&mut Fnc<'_>, Expr<B>) -> Expr<O>) -> Method<B, O>
where
    B: ?Sized,
    O: ?Sized,
{
    Method::Expr(e)
}

pub struct Vec2Methods<V, A, B, C> {
    pub xx: Method<V, A>,
    pub xy: Method<V, A>,
    pub yx: Method<V, A>,
    pub yy: Method<V, A>,
    pub xxx: Method<V, B>,
    pub xxy: Method<V, B>,
    pub xyx: Method<V, B>,
    pub xyy: Method<V, B>,
    pub yxx: Method<V, B>,
    pub yxy: Method<V, B>,
    pub yyx: Method<V, B>,
    pub yyy: Method<V, B>,
    pub xxxx: Method<V, C>,
    pub xxxy: Method<V, C>,
    pub xxyx: Method<V, C>,
    pub xxyy: Method<V, C>,
    pub xyxx: Method<V, C>,
    pub xyxy: Method<V, C>,
    pub xyyx: Method<V, C>,
    pub xyyy: Method<V, C>,
    pub yxxx: Method<V, C>,
    pub yxxy: Method<V, C>,
    pub yxyx: Method<V, C>,
    pub yxyy: Method<V, C>,
    pub yyxx: Method<V, C>,
    pub yyxy: Method<V, C>,
    pub yyyx: Method<V, C>,
    pub yyyy: Method<V, C>,
}

const fn vec2_methods<V, A, B, C>() -> Vec2Methods<V, A, B, C> {
    Vec2Methods {
        xx: swizzle(sw!(x x)),
        xy: swizzle(sw!(x y)),
        yx: swizzle(sw!(y x)),
        yy: swizzle(sw!(y y)),
        xxx: swizzle(sw!(x x x)),
        xxy: swizzle(sw!(x x y)),
        xyx: swizzle(sw!(x y x)),
        xyy: swizzle(sw!(x y y)),
        yxx: swizzle(sw!(y x x)),
        yxy: swizzle(sw!(y x y)),
        yyx: swizzle(sw!(y y x)),
        yyy: swizzle(sw!(y y y)),
        xxxx: swizzle(sw!(x x x x)),
        xxxy: swizzle(sw!(x x x y)),
        xxyx: swizzle(sw!(x x y x)),
        xxyy: swizzle(sw!(x x y y)),
        xyxx: swizzle(sw!(x y x x)),
        xyxy: swizzle(sw!(x y x y)),
        xyyx: swizzle(sw!(x y y x)),
        xyyy: swizzle(sw!(x y y y)),
        yxxx: swizzle(sw!(y x x x)),
        yxxy: swizzle(sw!(y x x y)),
        yxyx: swizzle(sw!(y x y x)),
        yxyy: swizzle(sw!(y x y y)),
        yyxx: swizzle(sw!(y y x x)),
        yyxy: swizzle(sw!(y y x y)),
        yyyx: swizzle(sw!(y y y x)),
        yyyy: swizzle(sw!(y y y y)),
    }
}

impl Methods for Vec2 {
    type Methods = Vec2Methods<Self, Self, Vec3, Vec4>;
    const METHODS: Vec2Methods<Self, Self, Vec3, Vec4> = vec2_methods();
}

impl Methods for IVec2 {
    type Methods = Vec2Methods<Self, Self, IVec3, IVec4>;
    const METHODS: Vec2Methods<Self, Self, IVec3, IVec4> = vec2_methods();
}

impl Methods for UVec2 {
    type Methods = Vec2Methods<Self, Self, UVec3, UVec4>;
    const METHODS: Vec2Methods<Self, Self, UVec3, UVec4> = vec2_methods();
}
