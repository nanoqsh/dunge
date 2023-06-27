use crate::shader_data::lights::UpdateError as LightsUpdateError;

/// The main loop error.
#[derive(Debug)]
pub enum Error {
    /// Returns when a rendered resourse or selected object not found.
    NotFound,

    /// Returns when trying to create too many objects.
    TooManyObjects,

    /// Returned when too large buffer is passed.
    TooLargeSize,

    /// Returns when globals is not set.
    GlobalsNotSet,

    /// Returns when textures is not set.
    TexturesNotSet,

    /// Returns when lights is not set.
    LightsNotSet,

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

/// Returned when too large buffer is passed.
#[derive(Debug)]
pub struct TooLargeSize;

impl From<TooLargeSize> for Error {
    fn from(_: TooLargeSize) -> Self {
        Self::TooLargeSize
    }
}

#[derive(Debug)]
pub enum SourceError {
    Source(LightsUpdateError),
    ResourceNotFound,
}

impl From<LightsUpdateError> for SourceError {
    fn from(v: LightsUpdateError) -> Self {
        Self::Source(v)
    }
}

impl From<ResourceNotFound> for SourceError {
    fn from(_: ResourceNotFound) -> Self {
        Self::ResourceNotFound
    }
}

#[derive(Debug)]
pub enum TexturesError {
    TooLargeSize,
    ResourceNotFound,
}

impl From<TooLargeSize> for TexturesError {
    fn from(_: TooLargeSize) -> Self {
        Self::TooLargeSize
    }
}

impl From<ResourceNotFound> for TexturesError {
    fn from(_: ResourceNotFound) -> Self {
        Self::ResourceNotFound
    }
}
