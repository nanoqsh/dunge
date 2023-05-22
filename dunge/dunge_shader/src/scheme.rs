use crate::{
    elements::*,
    nodes::{Binding, Location},
    out::Out,
    templater::Templater,
};

#[must_use]
pub fn generate(Scheme { vert, camera }: Scheme) -> Shader {
    let vert_input = VertexInput {
        fragment: &vert.fragment,
        pos: vert.dimension,
    };

    let vert_output = VertexOutput {
        fragment: &vert.fragment,
        world: true,
    };

    let types = {
        let mut out = Out::new();
        camera.define_type(&mut out);

        let mut location = Location::new();
        vert_input.define_type(&mut location, &mut out);
        InstanceInput::define_type(&mut location, &mut out);

        let mut location = Location::new();
        vert_output.define_type(&mut location, &mut out);

        out
    };

    let groups = {
        let mut out = Out::new();
        let mut binding = Binding::with_group(0);
        camera.declare_group(&mut binding, &mut out);

        let mut binding = Binding::with_group(1);
        Texture::declare_group(&mut binding, &mut out);

        out
    };

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
    pub fragment: Fragment,
}

pub struct Shader {
    pub src: String,
}
