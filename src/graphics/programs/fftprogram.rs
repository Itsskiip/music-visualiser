
use glium::implement_vertex;

use crate::graphics::{
    programs::{
        ProgramRunner,
        ShaderSrc,
        Decay,
    }, 
    Display,
};

#[derive(Default, Copy, Clone)]
struct FFTVertex {
    ampl: f32,
}
implement_vertex!(FFTVertex, ampl);



impl Decay<f32> for FFTVertex {
    fn assign(&mut self, rhs: f32) {
        self.ampl = (self.ampl + rhs) / 2.;
    }
}
pub struct FFTProgram { prog: ProgramRunner<f32, FFTVertex> }

impl FFTProgram {
    pub fn new(size: usize, display: &Display) -> Self {
        let shaders = ShaderSrc {
            vertex_shader: format!(r#"
                    #version 140
                    in float ampl;
            
                    void main() {{
                        gl_Position = vec4(gl_VertexID / {size}.0 - 0.5, log2(ampl * inversesqrt({size}.0)) / log2(20.0) / 10, 0.0, 1.0);
                    }}
                "#),
            fragment_shader: r#"
                    #version 140
                    out vec4 color;
            
                    void main() {
                        color = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                "#.to_string(),
            geometry_shader: None,
        };

        Self {
            prog: ProgramRunner::new(size, display, shaders)
        }
    }

    pub fn render(&mut self, values: &[f32]) {
        self.prog.render(
            values, 
            glium::index::NoIndices(glium::index::PrimitiveType::LineStrip), 
            &glium::uniforms::EmptyUniforms
        );
    }
}
