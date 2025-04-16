use std::marker::PhantomData;

use glium::{Frame, Program, Surface, VertexBuffer};
use crate::graphics::Display;
pub mod fftprogram;

#[derive(Clone)]
pub struct ShaderSrc {
    vertex_shader: String,
    fragment_shader: String,
    geometry_shader: Option<String>,
}

impl ShaderSrc {
    pub fn get_program(&self, display: &glium::Display<glium::glutin::surface::WindowSurface>) -> glium::Program {
        Program::from_source(display, 
            &self.vertex_shader, 
            &self.fragment_shader, 
            self.geometry_shader.as_ref().map(|x| x.as_str())
        ).unwrap()
    }
}
pub trait Decay<T> {
    fn assign(&mut self, rhs: T);
}
struct ProgramRunner<X, V>
where 
    X: Default + Clone + Copy, 
    V: Default + glium::Vertex,
    V: Decay<X>,
{
    program: Program,
    vertex_pre_buffer: Vec<V>,
    vertex_buffer: VertexBuffer<V>,

    _phantom: PhantomData<X>,
}

impl<X, V> ProgramRunner<X, V>
where 
    X: Default + Clone + Copy + std::fmt::Debug, 
    V: Default + glium::Vertex,
    V: Decay<X>,
{
    fn new(size: usize, display: &Display, shaders: ShaderSrc) -> Self {
        Self {
            program: shaders.get_program(display),
            vertex_pre_buffer: vec![V::default(); size],
            vertex_buffer: VertexBuffer::empty_dynamic(display, size).unwrap(),

            _phantom: PhantomData,
        }
    }
    
    pub fn render<'a, I: Into<glium::index::IndicesSource<'a>>, U:glium::uniforms::Uniforms>(&mut self, target: &mut Frame, values: &[X], indices: I, uniforms: &U) {
        self.vertex_pre_buffer.iter_mut().zip(values).for_each(|(v, x)| v.assign(*x));

        self.vertex_buffer.write(&self.vertex_pre_buffer);

        target.draw(
            &self.vertex_buffer, 
            indices, 
            &self.program, 
            uniforms, 
            &Default::default()
        ).unwrap();
    }
}
