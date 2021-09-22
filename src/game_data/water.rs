use gl_render::{data, buffer, uniform};
use resources::Resources;
use crate::camera::MVP;
use std::ffi::CString;
use failure::err_msg;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
}

impl From<(f32, f32, f32)> for Vertex {
    fn from(elem: (f32, f32, f32)) -> Self {
        Vertex { pos: elem.into() }
    }
}

pub struct Water {
    program: gl_render::Program,
    vbo: buffer::ArrayBuffer,
    ebo: buffer::ElementArrayBuffer,
    vao: buffer::VertexArray,
}

impl Water {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Water, failure::Error> {
        let program = gl_render::Program::from_res(gl, res, "shaders/water")?;

        let vertices: Vec<Vertex> = vec![
            (-0.7, 0.0,  0.7).into(), // bot left
            ( 0.7, 0.0,  0.7).into(), // bot right
            ( 0.7, 0.0, -0.7).into(), // top right
            (-0.7, 0.0, -0.7).into(), // top left
            ( 0.0, 0.5,  0.0).into(), // cone top
        ];

        let indices: Vec<u32> = vec![
            0, 1, 4,
            1, 2, 4,
            2, 3, 4,
            3, 0, 4,
        ];

        let vbo = buffer::ArrayBuffer::new(&gl);
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        let mut ebo = buffer::ElementArrayBuffer::new(&gl);
        ebo.bind();
        ebo.static_draw_data(&indices);
        ebo.set_elem_count(indices.len());
        ebo.unbind();

        let vao = buffer::VertexArray::new(&gl);
        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(&gl);
        ebo.bind();
        vbo.unbind();
        vao.unbind();
        ebo.unbind();

        Ok(Water {
            program,
            vbo,
            ebo,
            vao,
        })
    }

    pub fn render(&self, gl: &gl::Gl, mode: gl::types::GLenum) {
        self.program.use_it();
        self.vao.bind();

        unsafe {
            gl.DrawElements(
                mode,
                self.ebo.get_elem_count() as i32,
                gl::UNSIGNED_INT,
                0 as *const gl::types::GLvoid,
            )
        }
    }
}

impl uniform::HasUniform<MVP> for Water {
    fn apply_uniform(&self, gl: &gl::Gl, data: &MVP, name: &str) -> Result<(), failure::Error> {
        self.program.use_it();
        let name_cstr: CString = CString::new(name).map_err(err_msg)?;
        let matrix: *const f32 = data.get_transform().as_slice().as_ptr();
        unsafe {
            let location = gl.GetUniformLocation(self.program.id(), name_cstr.as_ptr() as *const i8);
            gl.UniformMatrix4fv(location, 1, gl::FALSE, matrix);
        }
        Ok(())
    }
}
