use crate::types::VectorType;

pub(crate) trait Vector {
    const TYPE: VectorType;
    type Scalar;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar);
}

impl Vector for glam::Vec2 {
    const TYPE: VectorType = VectorType::Vec2f;
    type Scalar = f32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::Vec3 {
    const TYPE: VectorType = VectorType::Vec3f;
    type Scalar = f32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::Vec3A {
    const TYPE: VectorType = VectorType::Vec3f;
    type Scalar = f32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::Vec4 {
    const TYPE: VectorType = VectorType::Vec4f;
    type Scalar = f32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::IVec2 {
    const TYPE: VectorType = VectorType::Vec2i;
    type Scalar = i32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::IVec3 {
    const TYPE: VectorType = VectorType::Vec3i;
    type Scalar = i32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::IVec4 {
    const TYPE: VectorType = VectorType::Vec4i;
    type Scalar = i32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::UVec2 {
    const TYPE: VectorType = VectorType::Vec2u;
    type Scalar = u32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::UVec3 {
    const TYPE: VectorType = VectorType::Vec3u;
    type Scalar = u32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl Vector for glam::UVec4 {
    const TYPE: VectorType = VectorType::Vec4u;
    type Scalar = u32;

    fn visit<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}
