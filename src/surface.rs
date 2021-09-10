use crate::gl_render::{self, buffer, data};
use crate::resources::Resources;
use std::ffi::{CString};
use crate::camera::MVP;
use failure::err_msg;
use gl_render::uniform;


#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
}

pub struct Surface {
    program: gl_render::Program,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    vao: buffer::VertexArray,
}

impl Surface {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Surface, failure::Error> {
        let program = gl_render::Program::from_res(gl, res, "surface")?;

        let vertices: Vec<Vertex> = vec![
            Vertex { pos: (-0.2, -0.2, 0.2).into() },
            Vertex { pos: ( 0.2, -0.2, 0.2).into() },
            Vertex { pos: ( 0.2,  0.2, 0.7).into() },
            Vertex { pos: (-0.2,  0.2, 0.7).into() },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            0, 2, 3,
        ];

        let vbo = buffer::ArrayBuffer::new(&gl);
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        let ebo = buffer::ElementArrayBuffer::new(&gl);
        ebo.bind();
        ebo.static_draw_data(&indices);
        ebo.unbind();

        let vao = buffer::VertexArray::new(&gl);
        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers(&gl);
        ebo.bind();
        vbo.unbind();
        vao.unbind();
        ebo.unbind();

        Ok(Surface {
            program,
            _vbo: vbo,
            _ebo: ebo,
            vao,
        })
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.program.use_it();
        self.vao.bind();

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                0 as *const gl::types::GLvoid,
            )
        }
    }
}

impl uniform::HasUniform<MVP> for Surface {
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