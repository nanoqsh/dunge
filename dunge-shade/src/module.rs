use {
    crate::irc::{Comp, Imports, Input, Irc, Stage},
    std::{iter, mem},
};

pub struct Info {
    pub(crate) vertex: &'static [VertexFormat],
    pub(crate) instance: &'static [VertexFormat],
    pub(crate) groups: [&'static [GroupFormat]; Self::MAX_GROUPS],
    pub(crate) group_stages: [u8; Self::MAX_GROUPS],
}

impl Info {
    pub(crate) const MAX_GROUPS: usize = 6;

    pub fn vertex(&self) -> impl ExactSizeIterator<Item = VertexFormat> {
        self.vertex.iter().copied()
    }

    pub fn instance(&self) -> impl ExactSizeIterator<Item = VertexFormat> {
        self.instance.iter().copied()
    }

    pub fn groups(&self) -> impl Iterator<Item = GroupInfo> {
        iter::zip(
            self.groups.iter().copied().take_while(|g| !g.is_empty()),
            self.group_stages,
        )
        .map(|(bindings, stages)| GroupInfo { bindings, stages })
    }
}

pub struct GroupInfo {
    bindings: &'static [GroupFormat],
    stages: u8,
}

impl GroupInfo {
    pub fn bindings(&self) -> impl Iterator<Item = GroupFormat> {
        self.bindings.iter().copied()
    }

    pub fn stages(&self) -> impl Iterator<Item = Stage> {
        Stage::from_bits(self.stages)
    }
}

pub struct Module {
    pub info: Info,
    pub naga: naga::Module,
    pub wgsl: String,
}

impl Module {
    fn new(info: Info, naga: naga::Module) -> Self {
        let wgsl;

        #[cfg(any(debug_assertions, feature = "wgsl"))]
        {
            use {
                naga::valid,
                std::{error::Error, fmt::Write},
            };

            let mut validator =
                valid::Validator::new(valid::ValidationFlags::all(), valid::Capabilities::empty());

            let info = match validator.validate(&naga) {
                Ok(info) => info,
                Err(e) => {
                    let mut inner = e.as_inner() as &dyn Error;
                    let mut s = format!("{inner}\n");
                    while let Some(source) = inner.source() {
                        _ = writeln!(&mut s, "{source}");
                        inner = source;
                    }

                    panic!("module error: {s}");
                }
            };

            #[cfg(feature = "wgsl")]
            {
                use naga::back::wgsl;

                wgsl = match wgsl::write_string(&naga, &info, wgsl::WriterFlags::all()) {
                    Ok(wgsl) => wgsl,
                    Err(e) => panic!("wgsl writer error: {e}"),
                };
            }

            #[cfg(not(feature = "wgsl"))]
            {
                _ = info;
            }
        }

        #[cfg(not(feature = "wgsl"))]
        {
            wgsl = String::new();
        }

        Self { info, naga, wgsl }
    }
}

pub(crate) type BuildFn = fn(&mut Irc, Boot) -> Comp<()>;

pub(crate) struct Make {
    boot: Boot,
    build: BuildFn,
}

impl Make {
    pub(crate) fn vertex(
        build: BuildFn,
        vertex_index: Option<usize>,
        instance_index: Option<usize>,
    ) -> Self {
        Self {
            boot: Boot {
                vertex_index,
                instance_index,
            },
            build,
        }
    }

    pub(crate) fn fragment(build: BuildFn) -> Self {
        Self {
            boot: Boot::default(),
            build,
        }
    }
}

pub(crate) fn make(info: Info, ms: &mut [Make]) -> Comp<Module> {
    let imports = Imports::default();
    let mut irc = Irc::new(imports);

    for make in ms {
        (make.build)(&mut irc, mem::take(&mut make.boot))?;
    }

    let naga = irc.build()?;
    Ok(Module::new(info, naga))
}

#[derive(Default)]
pub struct Boot {
    vertex_index: Option<usize>,
    instance_index: Option<usize>,
}

pub struct Hook<'irc> {
    irc: &'irc mut Irc,
    boot: Boot,
    count: usize,
}

impl<'irc> Hook<'irc> {
    pub(crate) fn new(irc: &'irc mut Irc, boot: Boot) -> Self {
        Self {
            irc,
            boot,
            count: 0,
        }
    }

    pub fn input<T>(&mut self)
    where
        T: Input,
    {
        if Some(self.count) == self.boot.vertex_index
            || Some(self.count) == self.boot.instance_index
        {
            T::init(self.irc);
        }

        self.count += 1;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Float32,
    Uint32,
    Sint32,
}

impl Format {
    pub(crate) const fn from_naga(s: naga::ScalarKind) -> Option<Self> {
        match s {
            naga::ScalarKind::Sint => Some(Self::Sint32),
            naga::ScalarKind::Uint => Some(Self::Uint32),
            naga::ScalarKind::Float => Some(Self::Float32),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VertexFormat {
    Scalar(Format),
    Vec2(Format),
    Vec3(Format),
    Vec4(Format),
}

#[derive(Clone, Copy, Debug)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
}

impl TextureDimension {
    pub(crate) const fn new(d: usize) -> Option<Self> {
        match d {
            1 => Some(Self::D1),
            2 => Some(Self::D2),
            3 => Some(Self::D3),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GroupFormat {
    Texture {
        dim: TextureDimension,
        scalar: Format,
    },
    Sampler,
    Uniform,
    Storage,
}
