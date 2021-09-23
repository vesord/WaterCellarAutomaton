use gl_render::{data, buffer, uniform};
use resources::Resources;
use crate::camera::MVP;
use std::ffi::CString;
use failure::err_msg;

extern crate chrono;
use chrono::prelude::*;

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

impl From<na::Vector3<f32>> for Vertex {
    fn from(v: na::Vector3<f32>) -> Self {
        Vertex { pos: (v.x, v.y, v.z).into() }
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
        self.vao.unbind();
        // println!("Render done");
    }

    pub fn set_grid(&mut self, grid_heights: &[Vec<f32>]) {
        let borders_h = grid_heights.len();
        self.grid = generate_borders(grid_heights, borders_h);
        let vertices = generate_vertex_grid(&self.grid);
        self.update_vbo(&vertices);
        let indices: Vec<u32> = vec![];
        self.update_ebo(&indices);
    }

    pub fn loop_add_water(&mut self, speed: f32) {    // TODO: probably remove it
        let mut new_water_level = self.water_level + speed;
        if new_water_level > 0.25 {
            new_water_level = 0.;
        }
        self.set_water_level(new_water_level);
    }

    pub fn decrease_water_level(&mut self) {
        let new_water_level = self.water_level - 0.05; // TODO: config
        self.set_water_level(new_water_level);
    }

    pub fn increase_water_level(&mut self) {
        let new_water_level = self.water_level + 0.05; // TODO: config
        self.set_water_level(new_water_level);
    }

    pub fn set_water_level(&mut self, water_level: f32) {
        let water_level = water_level.clamp(0., 1.);
        self.set_water_level_on_border(water_level);
        self.recalculate_render_data();
    }

    fn set_water_level_on_border(&mut self, water_level: f32) {
        let start = Utc::now();

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

        let end = Utc::now();
        println!("Set Water Level on grid done, taken {} ms", (end - start).num_milliseconds());
    }

    fn recalculate_render_data(&mut self) {
        let indices: Vec<u32> = generate_indices_for_render(&self.grid);
        self.update_ebo(&indices);
        self.update_vao();
    }

    pub fn print_col(&self, coord: (usize, usize)) {
        for elem in &self.grid[coord.0][coord.1] {
            print!("{:?} ", elem);
        }
    }

    fn update_vbo(&self, vertices: &[Vertex]) {
        self.vbo.bind();
        self.vbo.static_draw_data(vertices);
        self.vbo.unbind();
    }

    fn update_ebo(&mut self, indices: &[u32]) {
        self.ebo.bind();
        self.ebo.set_elem_count(indices.len());
        self.ebo.dynamic_draw_data(indices);
        self.ebo.unbind();
    }

    fn update_vao(&self) {
        self.vao.bind();
        self.vbo.bind();
        self.ebo.bind();
        self.vbo.unbind();
        self.vao.unbind();
        self.ebo.unbind();
    }

    fn update_buffers(&mut self, vertices: &[Vertex], indices: &[u32]) {
        let start = Utc::now();

        self.update_vbo(vertices);
        self.update_ebo(indices);
        self.update_vao();

        let end = Utc::now();
        println!("Opengl Buffers Update done, taken {} ms", (end - start).num_milliseconds());
    }
}

fn generate_borders(grid_heights: &[Vec<f32>], borders_h: usize) -> Vec<Vec<Vec<Drop>>> {
    let mut borders: Vec<Vec<Vec<Drop>>> = vec![];
    let step_h = 1. / (grid_heights.len() - 1) as f32;

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
    borders
}

fn generate_vertex_grid(cube: &Vec<Vec<Vec<Drop>>>) -> Vec<Vertex> {
    let start = Utc::now();

    let mut vertices: Vec<Vertex> = Vec::with_capacity(cube.len() * cube[0].len() * cube[0][0].len());
    let mut cur_coord = na::Vector3::new(-1., 0., -1.);
    let xz_step = 2. / (cube.len() - 1) as f32;
    let y_step = 1. / (cube[0][0].len() - 1) as f32;

    for side in cube {
        cur_coord.x = -1.;
        for col in side {
            cur_coord.y = 0.;

            // print!("Coord: {}, {}; Col: ", cur_coord.x, cur_coord.z);
            for _drop in col {
                // print!("{} ", cur_coord.y);
                vertices.push(cur_coord.into());
                cur_coord.y += y_step;
            }
            // println!();
            cur_coord.x += xz_step;
        }
        cur_coord.z += xz_step;
    }

    let end = Utc::now();
    println!("Gen Water Vertex Grid taken: {} ms", (end - start).num_milliseconds());
    vertices
}


fn generate_indices_for_render(cube: &Vec<Vec<Vec<Drop>>>) -> Vec<u32> {
    let start = Utc::now();

    let mut indices: Vec<u32> = Vec::with_capacity(cube.len().pow(3) * 6);

    let cols = cube[0].len();
    let drops = cube[0][0].len();
    let top_left_offset = (drops) as u32;
    let bot_left_offset = (drops * (cols + 1)) as u32;
    let bot_right_offset = (drops * cols) as u32;
    let mut cur_index: u32 = 0;

    for side in cube {
        for col in side {
            for drop in col {
                match drop {
                    Drop::Water => {
                        indices.push(cur_index);
                        indices.push((cur_index + bot_left_offset));
                        indices.push((cur_index + top_left_offset));

                        indices.push(cur_index);
                        indices.push((cur_index + bot_right_offset));
                        indices.push((cur_index + bot_left_offset));
                    }
                    _ => ()
                }
                cur_index += 1;
            }
        }
    }

    let end = Utc::now();
    println!("Water Indices Recalculation done, taken {} ms", (end - start).num_milliseconds());
    indices
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
