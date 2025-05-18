
use glium::{implement_vertex, uniforms::Uniforms, Frame};

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

struct FFTUniform {
    colour: [f32; 3],
}

impl Uniforms for FFTUniform {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut f: F) {
        f("colour", glium::uniforms::UniformValue::Vec3(self.colour));
    }
}


impl Decay<f32> for FFTVertex {
    fn assign(&mut self, rhs: f32) {
        self.ampl = if rhs > self.ampl { rhs } else {(self.ampl + rhs) / 2.};
    }
}
pub struct FFTProgram { 
    prog: ProgramRunner<f32, FFTVertex>, 
    uniforms: FFTUniform
}

impl FFTProgram {
    pub fn new(size: usize, display: &Display, colour: [f32; 3]) -> Self {
        let uniforms= FFTUniform { colour };
        let shaders = ShaderSrc {
            vertex_shader: format!(r#"
                    #version 140
                    in float ampl;
            
                    void main() {{
                        gl_Position = vec4((gl_VertexID / {size}.0 - 0.5) * 1.8, log2(ampl * inversesqrt({size}.0)) / log2(20.0) / 10, 0.0, 1.0);
                    }}
                "#),
            fragment_shader: r#"
                    #version 140
                    out vec4 color;
                    uniform vec3 colour;

                    void main() {
                        color = vec4(colour, 1.0);
                    }
                "#.to_string(),
            geometry_shader: None,
        };

        Self {
            prog: ProgramRunner::new(size, display, shaders),
            uniforms
        }
    }

    pub fn render(&mut self, target: &mut Frame, values: &[f32]) {
        self.prog.render(
            target,
            values, 
            glium::index::NoIndices(glium::index::PrimitiveType::LineStrip), 
            &self.uniforms
        );
    }
}
