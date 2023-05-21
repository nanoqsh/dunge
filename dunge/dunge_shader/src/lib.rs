mod elements;
mod nodes;
mod out;
mod templater;

use crate::{
    elements::*,
    nodes::{Binding, Location},
    out::Out,
    templater::Templater,
};

#[must_use]
pub fn generate(Scheme { vert, camera }: Scheme) -> Shader {
    let vert_input = VertexInput {
        pos: vert.dimension,
        col: vert.color,
    };

    let vert_output = VertexOutput {
        col: vert.color,
        world: vert.world,
    };

    let mut types = Out::new();
    camera.define_type(&mut types);

    let mut location = Location::new();
    vert_input.define_type(&mut location, &mut types);
    InstanceInput::define_type(&mut location, &mut types);

    let mut location = Location::new();
    vert_output.define_type(&mut location, &mut types);

    let mut binding = Binding::with_group(0);
    let mut groups = Out::new();
    camera.declare_group(&mut binding, &mut groups);

    let mut vertex_out = Out::new();
    vert_output.calc_vertex(vert_input, camera, &mut vertex_out);

    let mut fragment_col = Out::new();
    vert_output.calc_fragment(&mut fragment_col);

    Shader {
        src: Templater::default()
            .insert("types", types.buf())
            .insert("groups", groups.buf())
            .insert("vertex_out", vertex_out.buf())
            .insert("fragment_col", fragment_col.buf())
            .format(include_str!("../template.wgsl"))
            .expect("generate shader"),
    }
}

#[derive(Clone, Copy)]
pub struct Scheme {
    pub vert: Vertex,
    pub camera: Camera,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub color: Color,
    pub world: bool,
}

pub struct Shader {
    pub src: String,
}
