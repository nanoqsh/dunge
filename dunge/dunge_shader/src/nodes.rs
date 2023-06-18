use std::fmt;

pub(crate) struct Struct {
    pub name: &'static str,
    pub fields: Vec<Field>,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { name, fields } = self;
        writeln!(f, "struct {name} {{")?;
        for field in fields {
            writeln!(f, "{field}")?;
        }
        writeln!(f, "}}")?;
        writeln!(f)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Field {
    pub location: Location,
    pub name: Name,
    pub ty: Type,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Self { location, name, ty } = self;
        write!(f, "    {location}{name}: {ty},")
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Location {
    None,
    Num(u8),
    Position,
}

impl Location {
    pub fn new() -> Self {
        Self::Num(0)
    }

    pub fn next(&mut self) -> Self {
        let old = *self;
        if let Self::Num(n) = self {
            *n += 1;
        }

        old
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Num(n) => write!(f, "@location({n}) "),
            Self::Position => write!(f, "@builtin(position) "),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Binding {
    group: u32,
    binding: u32,
}

impl Binding {
    pub fn with_group(group: u32) -> Self {
        Self { group, binding: 0 }
    }

    pub fn next(&mut self) -> Self {
        let old = *self;
        self.binding += 1;
        old
    }

    pub fn get(self) -> u32 {
        self.binding
    }

    pub fn group(self) -> u32 {
        self.group
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Var {
    pub binding: Binding,
    pub uniform: bool,
    pub name: Name,
    pub ty: Type,
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Self {
            binding: Binding { group, binding },
            uniform,
            name,
            ty,
        } = self;

        writeln!(
            f,
            "@group({group}) @binding({binding}) var{spec} {name}: {ty};",
            spec = if uniform { "<uniform>" } else { "" },
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Name {
    Str(&'static str),
    Num { str: &'static str, n: u32 },
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Str(str) => write!(f, "{str}"),
            Self::Num { str, n } => write!(f, "{str}_{n}"),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Type {
    Simple(&'static str),
    Array { ty: &'static Self, size: u8 },
}

impl Type {
    pub const F32: Self = Self::Simple("f32");
    pub const U32: Self = Self::Simple("u32");
    pub const VEC2: Self = Self::Simple("vec2<f32>");
    pub const VEC3: Self = Self::Simple("vec3<f32>");
    pub const VEC4: Self = Self::Simple("vec4<f32>");
    pub const MAT4: Self = Self::Simple("mat4x4<f32>");
    pub const TEXTURE2D: Self = Self::Simple("texture_2d<f32>");
    pub const SAMPLER: Self = Self::Simple("sampler");
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Simple(ty) => write!(f, "{ty}"),
            Self::Array { ty, size } => write!(f, "array<{ty}, {size}>"),
        }
    }
}
