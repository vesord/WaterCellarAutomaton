use crate::gl_render::{self, buffer, data};
use crate::resources::Resources;
use std::ffi::{CString};
use crate::camera::MVP;
use failure::err_msg;
use gl_render::uniform;
use std::fmt::{Display, Formatter};


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

impl Display for Vertex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}), ", self.pos.d0, self.pos.d1, self.pos.d2)
    }
}

pub struct Surface {
    program: gl_render::Program,
    vbo: buffer::ArrayBuffer,
    ebo: buffer::ElementArrayBuffer,
    vao: buffer::VertexArray,
}

impl Surface {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Surface, failure::Error> {
        let program = gl_render::Program::from_res(gl, res, "shaders/surface")?;

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

        Ok(Surface {
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

    fn update_buffers(&mut self, vertices: &[Vertex], indices: &[u32]) {
        self.vbo.bind();
        self.vbo.static_draw_data(vertices);
        self.vbo.unbind();

        self.ebo.bind();
        self.ebo.set_elem_count(indices.len());
        self.ebo.static_draw_data(&indices);
        self.ebo.unbind();
    }

    pub fn set_grid(&mut self, grid: &[Vec<f32>]) -> Result<(), failure::Error> {
        let vertices: Vec<Vertex> = generate_vertex_grid(grid)?;
        let indices: Vec<u32> = generate_indices(grid.len())?;
        self.update_buffers(&vertices, &indices);
        Ok(())
    }
}

fn generate_vertex_grid(grid: &[Vec<f32>]) -> Result<Vec<Vertex>, failure::Error> {
    assert!(grid.len() > 1);

    let step = 2. / (grid.len() - 1) as f32;
    let mut coord: (f32, f32) = (-1. - step, -1. - step);   // (x, -z)
    let mut vertices: Vec<Vertex> = vec![];

    for row in grid {
        assert_eq!(row.len(), grid.len());
        coord.1 += step;
        for elem in row {
            coord.0 += step;
            vertices.push((coord.0, *elem, coord.1).into());
        }
        coord.0 = -1. - step;
    }
    Ok(vertices)
}

fn generate_indices(grid_size: usize) -> Result<Vec<u32>, failure::Error> {
    let mut indices: Vec<u32> = vec![];
    for i in 0..(grid_size - 1) {
        for j in 0..(grid_size - 1) {
            indices.push((i * grid_size + j) as u32);
            indices.push((i * grid_size + j + 1) as u32);
            indices.push((i * grid_size + j + 1 + grid_size) as u32);
            indices.push((i * grid_size + j) as u32);
            indices.push((i * grid_size + j + grid_size) as u32);
            indices.push((i * grid_size + j + grid_size + 1) as u32);
        }
    }
    Ok(indices)
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
