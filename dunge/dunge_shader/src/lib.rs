mod elements;
mod nodes;
mod out;
mod scheme;
mod templater;

pub use crate::{
    elements::{Camera, Color, Dimension, Fragment},
    scheme::{generate, Scheme, Shader, Vertex},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() {
        let shader = super::generate(Scheme {
            vert: Vertex {
                dimension: Dimension::D3,
                fragment: Fragment {
                    fixed_color: Some(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                    }),
                    vertex_color: true,
                    vertex_texture: true,
                },
            },
            camera: Camera::View,
        });

        std::fs::write("out.wgsl", shader.src).expect("write shader");
    }
}
