use {crate::layout::Plain, wgpu::PrimitiveTopology};

pub struct TopologyValue(PrimitiveTopology);

impl TopologyValue {
    pub(crate) fn into_inner(self) -> PrimitiveTopology {
        let Self(value) = self;
        value
    }
}

pub trait Topology {
    type Face: Clone + Plain;
    const VALUE: TopologyValue;
}

#[derive(Clone, Copy)]
pub struct PointList;

impl Topology for PointList {
    type Face = u16;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::PointList);
}

#[derive(Clone, Copy)]
pub struct LineList;

impl Topology for LineList {
    type Face = [u16; 2];
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::LineList);
}

#[derive(Clone, Copy)]
pub struct LineStrip;

impl Topology for LineStrip {
    type Face = u16;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::LineStrip);
}

#[derive(Clone, Copy)]
pub struct TriangleList;

impl Topology for TriangleList {
    type Face = [u16; 3];
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::TriangleList);
}

#[derive(Clone, Copy)]
pub struct TriangleStrip;

impl Topology for TriangleStrip {
    type Face = u16;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::TriangleStrip);
}
