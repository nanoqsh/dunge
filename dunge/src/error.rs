/// The error indicating some object is not set.
#[derive(Debug)]
pub enum NotSet {
    /// Returns when globals is not set.
    Globals,

    /// Returns when textures is not set.
    Textures,

    /// Returns when lights is not set.
    Lights,

    /// Returns when spaces is not set.
    Spaces,
}

/// Returned when an invalid buffer size is provided.
#[derive(Debug)]
pub struct InvalidSize;
