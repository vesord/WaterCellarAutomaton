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

#[derive(Debug)]
enum Drop {
    Empty,
    Border,
    Water,
}

pub struct Water {
    water_level: f32,
    grid: Vec<Vec<Vec<Drop>>>,
    program: gl_render::Program,
    vbo: buffer::ArrayBuffer,
    ebo: buffer::ElementArrayBuffer,
    vao: buffer::VertexArray,
}

impl Water {
    pub fn new(res: &Resources, gl: &gl::Gl) -> Result<Water, failure::Error> {
        let program = gl_render::Program::from_res(gl, res, "shaders/water")?;

        let vertices: Vec<Vertex> = vec![
            (-1.0, 0.1,  1.0).into(), // bot left
            ( 1.0, 0.1,  1.0).into(), // bot right
            ( 1.0, 0.1, -1.0).into(), // top right
            (-1.0, 0.1, -1.0).into(), // top left
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2,
            2, 3, 0,
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

        let grid = vec![];
        let water_level: f32 = 0.;

        Ok(Water {
            water_level,
            grid,
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
        println!("Render done");
    }

    pub fn generate_borders(&mut self, grid_heights: &[Vec<f32>]) {
        let mut borders: Vec<Vec<Vec<Drop>>> = vec![];
        let borders_h: usize = grid_heights.len();
        let step_h = 1. / grid_heights.len() as f32;

        for row in grid_heights {
            let mut side: Vec<Vec<Drop>> = Vec::with_capacity(grid_heights.len());

            for elem in row {
                let mut col: Vec<Drop> = Vec::with_capacity(borders_h);
                let cur_height = (elem / step_h).ceil() as usize;
                for _i in 0..cur_height {
                    col.push(Drop::Border);
                }
                for _i in cur_height..grid_heights.len() {
                    col.push(Drop::Empty);
                }
                side.push(col);
            }
            borders.push(side);
        }
        self.grid = borders
    }

    pub fn loop_add_water(&mut self, speed: f32) -> Result<(), failure::Error> {
        let mut new_water_level = self.water_level + speed;
        if new_water_level > 0.25 {
            new_water_level = 0.;
        }
        self.set_water_level(new_water_level)
    }

    pub fn decrease_water_level(&mut self) -> Result<(), failure::Error> {
        let new_water_level = self.water_level - 0.05; // TODO: config
        self.set_water_level(new_water_level)
    }

    pub fn increase_water_level(&mut self) -> Result<(), failure::Error> {
        let new_water_level = self.water_level + 0.05; // TODO: config
        self.set_water_level(new_water_level)
    }

    pub fn set_water_level(&mut self, water_level: f32) -> Result<(), failure::Error> {
        let water_level = water_level.clamp(0., 1.);
        self.water_level = water_level;
        let step = 1. / self.grid.len() as f32;

        for side in &mut self.grid {
            for col in side {
                let mut cur_level: f32 = 0.;
                for mut drop in col {
                    *drop = match (cur_level, &drop) {
                        (_, Drop::Border) => Drop::Border,
                        (level, _) if level < water_level => Drop::Water,
                        (_, _) => Drop::Empty,
                    };
                    cur_level += step;
                }
            }
        }
        println!("Set Water Level done");

        self.recalculate_render_data()
    }

    fn recalculate_render_data(&mut self) -> Result<(), failure::Error> {
        let (vertices, indices): (Vec<Vertex>, Vec<u32>) = generate_vertices_indices_for_render(&self.grid)?;
        self.update_buffers(&vertices, &indices);
        Ok(())
    }

    pub fn print_col(&self, coord: (usize, usize)) {
        for elem in &self.grid[coord.0][coord.1] {
            print!("{:?} ", elem);
        }
    }

    fn update_buffers(&mut self, vertices: &[Vertex], indices: &[u32]) {
        self.vbo.bind();
        self.vbo.dynamic_draw_data(vertices);
        self.vbo.unbind();

        self.ebo.bind();
        self.ebo.set_elem_count(indices.len());
        self.ebo.dynamic_draw_data(indices);
        self.ebo.unbind();

        self.vao.bind();
        self.vbo.bind();
        self.ebo.bind();
        self.vbo.unbind();
        self.vao.unbind();
        self.ebo.unbind();
        println!("Opengl Buffers Update done");
    }
}

fn generate_vertices_indices_for_render(cube: &Vec<Vec<Vec<Drop>>>) -> Result<(Vec<Vertex>, Vec<u32>), failure::Error> {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(cube.len().pow(3) * 3);
    let mut indices: Vec<u32> = Vec::with_capacity(cube.len().pow(3) * 6);
    let mut cur_coord = (-1., 0., -1.);
    let xz_step = 2. / cube.len() as f32;
    let mut cur_index = 0;

    for side in cube {
        cur_coord.0 = -1.;
        for col in side {
            cur_coord.1 = 0.;
            let y_step = 1. / col.len() as f32;

            for drop in col {
                match drop {
                    Drop::Water => {
                        // println!("Pushed water at: {:?}", cur_coord);
                        vertices.push(cur_coord.into());
                        vertices.push((cur_coord.0 + xz_step, cur_coord.1, cur_coord.2).into());
                        vertices.push((cur_coord.0 + xz_step, cur_coord.1, cur_coord.2 + xz_step).into());
                        vertices.push((cur_coord.0, cur_coord.1, cur_coord.2 + xz_step).into());
                        indices.push(cur_index);
                        indices.push(cur_index + 2);
                        indices.push(cur_index + 1);
                        indices.push(cur_index);
                        indices.push(cur_index + 3);
                        indices.push(cur_index + 2);
                        cur_index += 4;
                    }
                    _ => ()
                }
                cur_coord.1 += y_step;
            }
            cur_coord.0 += xz_step;
        }
        cur_coord.2 += xz_step;
    }
    println!("Water Buffers Recalculation done");
    Ok((vertices, indices))
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
