//! Topology types and traits.

use {bytemuck::Pod, wgpu::PrimitiveTopology};

/// The topology type of the [topology](crate::topology::Topology) trait.
pub struct TopologyValue(PrimitiveTopology);

impl TopologyValue {
    pub(crate) fn into_inner(self) -> PrimitiveTopology {
        let Self(value) = self;
        value
    }
}

/// The topology trait. Specifies how the mesh is presented.
pub trait Topology {
    type Face: Pod;
    const N: usize;
    const VALUE: TopologyValue;
}

/// Represents a vertex data as a list of points.
#[derive(Clone, Copy)]
pub struct PointList;

impl Topology for PointList {
    type Face = u16;
    const N: usize = 1;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::PointList);
}

/// Represents a vertex data as a list of lines.
#[derive(Clone, Copy)]
pub struct LineList;

impl Topology for LineList {
    type Face = [u16; Self::N];
    const N: usize = 2;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::LineList);
}

/// Represents a vertex data as a line strip.
#[derive(Clone, Copy)]
pub struct LineStrip;

impl Topology for LineStrip {
    type Face = u16;
    const N: usize = 1;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::LineStrip);
}

/// Represents a vertex data as a list of triangles.
#[derive(Clone, Copy)]
pub struct TriangleList;

impl Topology for TriangleList {
    type Face = [u16; Self::N];
    const N: usize = 3;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::TriangleList);
}

/// Represents a vertex data as a triangle strip.
#[derive(Clone, Copy)]
pub struct TriangleStrip;

impl Topology for TriangleStrip {
    type Face = u16;
    const N: usize = 1;
    const VALUE: TopologyValue = TopologyValue(PrimitiveTopology::TriangleStrip);
}
