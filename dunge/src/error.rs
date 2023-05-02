/// The main loop error.
#[derive(Debug)]
pub enum Error {
    /// Returns when a rendered resourse or selected object not found.
    NotFound,

    /// Returns when trying to create too many objects.
    TooManyObjects,

    /// Returns when an instance of rendered resourse is not set.
    InstanceNotSet,
}

/// Returns when a rendered resourse not found.
#[derive(Debug)]
pub struct ResourceNotFound;

impl From<ResourceNotFound> for Error {
    fn from(_: ResourceNotFound) -> Self {
        Self::NotFound
    }
}

/// Returns when trying to create too many light sources.
#[derive(Debug)]
pub struct TooManySources;

impl From<TooManySources> for Error {
    fn from(_: TooManySources) -> Self {
        Self::TooManyObjects
    }
}

/// Returns when a selected light source not found.
#[derive(Debug)]
pub struct SourceNotFound;

impl From<SourceNotFound> for Error {
    fn from(_: SourceNotFound) -> Self {
        Self::NotFound
    }
}

/// Returns when trying to create too many light spaces.
#[derive(Debug)]
pub struct TooManySpaces;

impl From<TooManySpaces> for Error {
    fn from(_: TooManySpaces) -> Self {
        Self::TooManyObjects
    }
}

/// Returns when a selected light space not found.
#[derive(Debug)]
pub struct SpaceNotFound;

impl From<SpaceNotFound> for Error {
    fn from(_: SpaceNotFound) -> Self {
        Self::NotFound
    }
}
