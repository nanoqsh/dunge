pub trait Topology {
    type Face: Clone;
}

#[derive(Clone, Copy)]
pub struct PointList;

impl Topology for PointList {
    type Face = u16;
}

#[derive(Clone, Copy)]
pub struct LineList;

impl Topology for LineList {
    type Face = [u16; 2];
}

#[derive(Clone, Copy)]
pub struct LineStrip;

impl Topology for LineStrip {
    type Face = u16;
}

#[derive(Clone, Copy)]
pub struct TriangleList;

impl Topology for TriangleList {
    type Face = [u16; 3];
}

#[derive(Clone, Copy)]
pub struct TriangleStrip;

impl Topology for TriangleStrip {
    type Face = u16;
}
