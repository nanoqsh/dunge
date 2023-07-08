/// The main loop error.
#[derive(Debug)]
pub enum Error {
    /// Returns when a rendered resourse or selected object not found.
    NotFound,

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

/// Returned when too large buffer is passed.
#[derive(Debug)]
pub struct TooLargeSize;
