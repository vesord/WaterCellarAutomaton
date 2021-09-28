use gl_render::{data, buffer, uniform};
use resources::Resources;
use crate::camera::MVP;
use std::ffi::CString;
use failure::err_msg;

extern crate chrono;
use chrono::prelude::*;
use std::ops::{Index, IndexMut};

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

#[repr(C, packed)]
struct TriangleIdx {
    i0: u32,
    i1: u32,
    i2: u32,
}

impl From<(u32, u32, u32)> for TriangleIdx {
    fn from(other: (u32, u32, u32)) -> Self {
        TriangleIdx {
            i0: other.0, i1: other.1, i2: other.2,
        }
    }
}

impl TriangleIdx {
    pub fn move_down(&mut self) {
        self.i0 -= 1;
        self.i1 -= 1;
        self.i2 -= 1;
    }

    pub fn move_north(&mut self) {

    }
}

const POINTS_PER_PARTICLE: usize = 6;

#[repr(C, packed)]
struct ParticleShape {
    t0: TriangleIdx,
    t1: TriangleIdx,
}

impl ParticleShape {
    pub fn new(x: u32, y: u32, z: u32, xz_size: u32, y_size: u32) -> ParticleShape {
        let p0 = z * xz_size * y_size + x * xz_size + y;    // Top left
        let p1  = p0 + y_size;                              // Top right
        let p2 = p0 + y_size * (xz_size + 1);               // Bot right
        let p3 = p0 + y_size * xz_size;                     // Bot left

        ParticleShape {
            t0: (p0, p1, p2).into(),
            t1: (p0, p3, p2).into(),
        }
    }

    pub fn move_down(&mut self) {
        self.t0.move_down();
        self.t1.move_down();
    }

    pub fn move_north(&mut self) {
        self.t0.move_north();
        self.t1.move_north();
    }

    pub fn move_south(&mut self) {
        self.t0.move_south();
        self.t1.move_south();
    }

    pub fn move_west(&mut self) {
        self.t0.move_west();
        self.t1.move_west();
    }

    pub fn move_east(&mut self) {
        self.t0.move_east();
        self.t1.move_east();
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
enum Particle {
    Empty,
    Border,
    Water,
}

pub struct Water {
    water_level_max: usize,
    water_level: usize,
    grid: Vec<Vec<Vec<Particle>>>,
    locations: Vec<na::Vector3<usize>>,
    ib_data: Vec<ParticleShape>,
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
        let water_level = 0;
        let water_level_max = 0;
        let locations = vec![];
        let ib_data = vec![];

        Ok(Water {
            water_level_max, water_level,
            grid, locations, ib_data,
            program, vbo, ebo, vao,
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
        self.water_level_max = borders_h; // TODO: add xyz_size to config
        let vertices = generate_vertex_grid(&self.grid);
        self.update_vbo(&vertices);

        for z in 0..5 {
            for i in 0..200 {
                for j in 0..200 {
                    add_particle(&mut self.locations, &mut self.ib_data, i, 100 + z, j, 200, 200);
                }
            }
        }


        self.update_ebo();
        self.update_vao();
    }

    pub fn loop_add_water(&mut self) {
        match self.water_level {
            level if level + 1 >= self.water_level_max / 2 => self.flush(),
            _ => self.increase_water_level(),
        }
    }

    pub fn increase_water_level(&mut self) {
        let new_water_level = (self.water_level + 1).clamp(0, self.water_level_max - 1); // TODO: config
        self.water_level = new_water_level;
        self.fill_water_level(new_water_level);
    }

    pub fn flush(&mut self) {
        self.water_level = 0;
        self.ib_data.clear();
        for loc in &self.locations {
            self.grid[loc.z][loc.x][loc.y] = Particle::Empty; // TODO: think how to optimize
        }
        self.locations.clear();
        self.water_level = 0;

        // for side in &mut self.grid {
        //     for col in side {
        //         for drop in col {
        //             *drop = match drop {
        //                 Drop::Water => Drop::Empty,
        //                 Drop::Empty => Drop::Empty,
        //                 Drop::Border => Drop::Border,
        //             }
        //         }
        //     }
        // }

        self.update_ebo();
        self.update_vao();
    }

    pub fn modulate(&mut self) {
        let start = Utc::now();

        for (loc, square) in self.locations.iter_mut().zip(&mut self.ib_data) {
            let x = loc.x.clamp(1, 199);
            let y = loc.y.clamp(1, 199);    // TODO: add to config
            let z = loc.z.clamp(1, 199);

            if self.grid[z][x][y - 1] == Particle::Empty {  // Bottom
                self.grid[z][x][y] = Particle::Empty;
                self.grid[z][x][y - 1] = Particle::Water;
                loc.y = loc.y - 1;
                square.move_down();
            }
            else if self.grid[z - 1][x][y] == Particle::Empty {
                self.grid[z][x][y] = Particle::Empty;
                self.grid[z - 1][x][y] = Particle::Water;
                loc.z = loc.z - 1;
                square.move_north();
            }
            else if self.grid[z + 1][x][y] == Particle::Empty {
                self.grid[z][x][y] = Particle::Empty;
                self.grid[z + 1][x][y] = Particle::Water;
                loc.z = loc.z + 1;
                square.move_south();
            }
            else if self.grid[z][x - 1][y] == Particle::Empty {
                self.grid[z][x][y] = Particle::Empty;
                self.grid[z][x - 1][y] = Particle::Water;
                loc.x = loc.x - 1;
                square.move_west();
            }
            else if self.grid[z][x + 1][y] == Particle::Empty {
                self.grid[z][x][y] = Particle::Empty;
                self.grid[z][x + 1][y] = Particle::Water;
                loc.x = loc.x + 1;
                square.move_east();
            }
        }

        self.update_ebo();
        self.update_vao();

        let end = Utc::now();

        println!("Modulation done, elems: {}, time {} ms", self.locations.len(), (end-start).num_milliseconds());
    }

    fn fill_water_level(&mut self, level: usize) {
        let start = Utc::now();

        let xz_size = self.grid.len() as u32; // TODO: add xyz_size to config
        let y_size = self.water_level_max as u32; // TODO: add xyz_size to config
        let mut cur_water_idx_x = 0;
        let mut cur_water_idx_z = 0;

        for side in self.grid.split_last_mut().unwrap().1 {     // unwrap assumes self.grid is not empty
            cur_water_idx_x = 0;
            for col in side.split_last_mut().unwrap().1 {
                *col.index_mut(level) = match col.index(level) {
                    Particle::Empty => {
                        add_particle(&mut self.locations, &mut self.ib_data,
                                     cur_water_idx_x, level, cur_water_idx_z,
                                     xz_size, y_size);        // TODO: add xyz_size to config
                        Particle::Water
                    },
                    Particle::Water => Particle::Water,
                    Particle::Border => Particle::Border,
                };
                cur_water_idx_x += 1;
            }
            cur_water_idx_z += 1;
        }

        let end = Utc::now();

        // println!("Fill water level done, taken {} ms", (end - start).num_milliseconds());

        self.update_ebo();
        self.update_vao();
    }

    fn add_particle(&mut self, x: usize, y: usize, z: usize) {
        add_particle(&mut self.locations, &mut self.ib_data,
                     x, y, z,
                     self.grid.len() as u32, self.water_level_max as u32);
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

    fn update_ebo(&mut self) {
        self.ebo.bind();
        self.ebo.dynamic_draw_data(&self.ib_data);
        self.ebo.set_elem_count(self.ib_data.len() * POINTS_PER_PARTICLE);
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
}

fn add_particle(locations: &mut Vec<na::Vector3<usize>>, ib_data: &mut Vec<ParticleShape>,
                x: usize, y: usize, z: usize,
                xz_size: u32, y_size: u32) {
    locations.push(na::Vector3::new(x, y, z));
    ib_data.push(ParticleShape::new(
        x as u32,
        y as u32,
        z as u32,
        xz_size,
        y_size)
    );
}

fn generate_borders(grid_heights: &[Vec<f32>], borders_h: usize) -> Vec<Vec<Vec<Particle>>> {
    let mut borders: Vec<Vec<Vec<Particle>>> = vec![];
    let step_h = 1. / (grid_heights.len() - 1) as f32;

    for row in grid_heights {
        let mut side: Vec<Vec<Particle>> = Vec::with_capacity(grid_heights.len());

        for elem in row {
            let mut col: Vec<Particle> = Vec::with_capacity(borders_h);
            let cur_height = (elem / step_h).ceil() as usize;
            for _i in 0..cur_height {
                col.push(Particle::Border);
            }
            for _i in cur_height..grid_heights.len() {
                col.push(Particle::Empty);
            }
            side.push(col);
        }
        borders.push(side);
    }
    borders
}

fn generate_vertex_grid(cube: &Vec<Vec<Vec<Particle>>>) -> Vec<Vertex> {
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
