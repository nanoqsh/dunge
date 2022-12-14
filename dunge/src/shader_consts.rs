pub(crate) struct Location {
    pub(crate) group: u32,
    pub(crate) binding: u32,
}

pub(crate) mod color {
    use super::Location;

    pub(crate) const CAMERA: Location = Location {
        group: 0,
        binding: 0,
    };
}

pub(crate) mod flat {
    use super::Location;

    #[allow(dead_code)]
    pub(crate) const T_DIFFUSE: Location = Location {
        group: 0,
        binding: 0,
    };

    pub(crate) const S_DIFFUSE: Location = Location {
        group: 0,
        binding: 1,
    };
}

pub(crate) mod post {
    use super::Location;

    pub(crate) const DATA: Location = Location {
        group: 0,
        binding: 0,
    };

    pub(crate) const T_DIFFUSE: Location = Location {
        group: 1,
        binding: 0,
    };

    pub(crate) const S_DIFFUSE: Location = Location {
        group: 1,
        binding: 1,
    };
}

pub(crate) mod textured {
    use super::Location;

    pub(crate) const CAMERA: Location = Location {
        group: 0,
        binding: 0,
    };

    pub(crate) const T_DIFFUSE: Location = Location {
        group: 1,
        binding: 0,
    };

    pub(crate) const S_DIFFUSE: Location = Location {
        group: 1,
        binding: 1,
    };
}

pub(crate) const VERTEX_BUFFER_SLOT: u32 = 0;
pub(crate) const INSTANCE_BUFFER_SLOT: u32 = 1;
