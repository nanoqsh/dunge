use std::ops;

#[derive(Clone, Copy)]
pub enum Stage {
    Vertex,
    Fragment,
    Compute,
}

impl Stage {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Vertex => "vs",
            Self::Fragment => "fs",
            Self::Compute => "cs",
        }
    }

    pub(crate) fn shader_stage(self) -> naga::ShaderStage {
        match self {
            Self::Vertex => naga::ShaderStage::Vertex,
            Self::Fragment => naga::ShaderStage::Fragment,
            Self::Compute => naga::ShaderStage::Compute,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Stages(u8);

impl Stages {
    const VERTEX: Self = Self(1 << 0);
    const FRAGMENT: Self = Self(1 << 1);
    const COMPUTE: Self = Self(1 << 2);

    pub fn has(self, stage: Stage) -> bool {
        self.0 & Self::from(stage).0 != 0
    }
}

impl From<Stage> for Stages {
    fn from(stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self::VERTEX,
            Stage::Fragment => Self::FRAGMENT,
            Stage::Compute => Self::COMPUTE,
        }
    }
}

impl ops::BitOrAssign for Stages {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl ops::BitOr for Stages {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl ops::BitOrAssign<Stage> for Stages {
    fn bitor_assign(&mut self, rhs: Stage) {
        *self |= Self::from(rhs);
    }
}

impl ops::BitOr<Stage> for Stages {
    type Output = Self;

    fn bitor(mut self, rhs: Stage) -> Self::Output {
        self |= rhs;
        self
    }
}
