use crate::shader_data::{
    lights::UpdateError as LightsUpdateError, spaces::UpdateDataError as SpacesUpdateDataError,
};

/// The main loop error.
#[derive(Debug)]
pub enum Error {
    /// Returns when a rendered resourse or selected object not found.
    NotFound,

    /// Returned when too large buffer is passed.
    TooLargeSize,

    /// Returns when the requested object is not set.
    NotSet(NotSetError),
}

impl From<NotSetError> for Error {
    fn from(v: NotSetError) -> Self {
        Self::NotSet(v)
    }
}

/// The error indicating some object is not set.
#[derive(Debug)]
pub enum NotSetError {
    /// Returns when globals is not set.
    Globals,

    /// Returns when textures is not set.
    Textures,

    /// Returns when lights is not set.
    Lights,

    /// Returns when spaces is not set.
    Spaces,
}

/// Returns when a rendered resourse not found.
#[derive(Debug)]
pub struct ResourceNotFound;

impl From<ResourceNotFound> for Error {
    fn from(_: ResourceNotFound) -> Self {
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
    Update(LightsUpdateError),
    ResourceNotFound,
}

impl From<LightsUpdateError> for SourceError {
    fn from(v: LightsUpdateError) -> Self {
        Self::Update(v)
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

#[derive(Debug)]
pub enum SpaceError {
    Update(SpacesUpdateDataError),
    ResourceNotFound,
}

impl From<SpacesUpdateDataError> for SpaceError {
    fn from(v: SpacesUpdateDataError) -> Self {
        Self::Update(v)
    }
}

impl From<ResourceNotFound> for SpaceError {
    fn from(_: ResourceNotFound) -> Self {
        Self::ResourceNotFound
    }
}
