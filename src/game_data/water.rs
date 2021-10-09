extern crate chrono;
extern crate rand;

mod vertex;
mod particle_shape;

use vertex::Vertex;
use gl_render::{data, buffer, uniform};
use resources::Resources;
use crate::camera::MVP;
use std::ffi::CString;
use failure::err_msg;

use chrono::prelude::*;
use std::ops::{Index, IndexMut};
use crate::game_data::GRID_WIDTH;
use self::rand::Rng;
use std::borrow::Borrow;
use particle_shape::{ParticleShape, POINTS_PER_PARTICLE};


#[derive(Debug)]
#[derive(PartialEq)]
enum Particle {
    Empty,
    Border(Direction),
    Water(Direction),
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
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

const WATER_GRID_WIDTH: usize = GRID_WIDTH;
const WATER_GIRD_HEIGHT: usize = GRID_WIDTH / 2;
const WATER_RAIN_ITERATIONS: usize =
    ((WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as f32 * 0.0001) as usize + 1;

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
    }

    pub fn set_grid(&mut self, grid_heights: &[Vec<f32>]) {
        let borders_h = WATER_GIRD_HEIGHT;
        self.grid = generate_borders(grid_heights, borders_h);
        self.water_level_max = borders_h;
        let vertices = generate_vertex_grid(grid_heights, borders_h);
        self.update_vbo(&vertices);

        self.update_ebo();
        self.update_vao();
    }

    pub fn modulate(&mut self) {
        let start = Utc::now();
        let mut comparisons = 0;

        for (loc, square) in self.locations.iter_mut().zip(&mut self.ib_data) {
            let x = loc.x;
            let y = loc.y;
            let z = loc.z;
            let mut cur_dir =  match self.grid[z][x][y] {
                Particle::Water(dir) => dir,
                _ => Direction::East,
            };

            if loc.y < self.water_level {
                continue;
            }

            match self.grid[z][x][y - 1] {
                Particle::Empty => {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x][y - 1] = Particle::Water(cur_dir);
                    loc.y = loc.y - 1;
                    square.move_down();
                    continue ;
                }
                Particle::Border(dir) => {
                    cur_dir = dir;
                    self.grid[z][x][y] = Particle::Water(dir);
                }
                _ => ()
            }

            if cur_dir == Direction::North {
                if (z > 0) && (self.grid[z - 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z - 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z - 1;
                    square.move_north();
                }
                else if (x > 0) && (self.grid[z][x - 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x - 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x - 1;
                    square.move_west();
                }
                else if (x < WATER_GRID_WIDTH - 2) && (self.grid[z][x + 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x + 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x + 1;
                    square.move_east();
                }
            }
            else if cur_dir == Direction::South {
                if (z < WATER_GRID_WIDTH - 2) && (self.grid[z + 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z + 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z + 1;
                    square.move_south();
                }
                else if (x < WATER_GRID_WIDTH - 2) && (self.grid[z][x + 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x + 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x + 1;
                    square.move_east();
                }
                else if (x > 0) && (self.grid[z][x - 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x - 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x - 1;
                    square.move_west();
                }
            }
            else if cur_dir == Direction::East {
                if (x < WATER_GRID_WIDTH - 2) && (self.grid[z][x + 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x + 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x + 1;
                    square.move_east();
                }
                else if (z < WATER_GRID_WIDTH - 2) && (self.grid[z + 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z + 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z + 1;
                    square.move_south();
                }
                else if (z > 0) && (self.grid[z - 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z - 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z - 1;
                    square.move_north();
                }
            }
            else if cur_dir == Direction::West {
                if (x > 0) && (self.grid[z][x - 1][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z][x - 1][y] = Particle::Water(cur_dir);
                    loc.x = loc.x - 1;
                    square.move_west();
                }
                else if (z > 0) && (self.grid[z - 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z - 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z - 1;
                    square.move_north();
                }
                else if (z < WATER_GRID_WIDTH - 2) && (self.grid[z + 1][x][y] == Particle::Empty) {
                    self.grid[z][x][y] = Particle::Empty;
                    self.grid[z + 1][x][y] = Particle::Water(cur_dir);
                    loc.z = loc.z + 1;
                    square.move_south();
                }
            }
            comparisons += 1;
        }

        self.update_ebo();
        self.update_vao();

        self.update_water_level();

        let end = Utc::now();

        println!("Modulation done, elems: {}, comps: {}, time {} ms", self.locations.len(), comparisons, (end-start).num_milliseconds());
    }

    pub fn flush(&mut self) {
        self.water_level = 0;
        self.ib_data.clear();
        for loc in &self.locations {
            self.grid[loc.z][loc.x][loc.y] = Particle::Empty; // TODO: think how to optimize
        }
        self.locations.clear();
        self.water_level = 0;

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
        let new_water_level = (self.water_level + 1).clamp(0, self.water_level_max - 2); // TODO: config
        self.water_level = new_water_level;
        self.fill_water_level(new_water_level);
    }

    fn fill_water_level(&mut self, level: usize) {
        let xz_size = WATER_GRID_WIDTH as u32; // TODO: add xyz_size to config
        let y_size = WATER_GIRD_HEIGHT as u32; // TODO: add xyz_size to config
        let mut cur_water_idx_x = 0;
        let mut cur_water_idx_z = 0;

        for side in &mut self.grid {
            cur_water_idx_x = 0;
            for col in side {
                *col.index_mut(level) = match col.index(level) {
                    Particle::Empty => {
                        add_particle(&mut self.locations, &mut self.ib_data,
                                     cur_water_idx_x, level, cur_water_idx_z,
                                     xz_size, y_size);      // TODO: add xyz_size to config
                        Particle::Water(Direction::East)
                    },
                    Particle::Water(some) => Particle::Water(*some),
                    Particle::Border(some) => Particle::Border(*some),
                };
                cur_water_idx_x += 1;
            }
            cur_water_idx_z += 1;
        }
        self.update_ebo();
        self.update_vao();
    }

    fn update_water_level(&mut self) {
        let mut need_up = true;
        let cur_water_level = self.water_level;

        'outer: for side in &mut self.grid {
            for col in side {
                if col[cur_water_level] == Particle::Empty {
                    need_up = false;
                    break 'outer;
                }
            }
        }

        if need_up {
            self.water_level = std::cmp::min(cur_water_level + 1, self.water_level_max);
        }
    }

    pub fn add_rain_particles(&mut self) {
        for _i in 0..WATER_RAIN_ITERATIONS {
            let x = rand::thread_rng().gen_range(0..WATER_GRID_WIDTH - 2);
            let z = rand::thread_rng().gen_range(0..WATER_GRID_WIDTH - 2);
            let y   = WATER_GIRD_HEIGHT - 2;
            let dir = match rand::thread_rng().gen_range(0..3) {
                0 => Direction::West,
                1 => Direction::East,
                2 => Direction::North,
                _ => Direction::South
            };

            if self.grid[z][x][y] == Particle::Empty {
                self.grid[z][x][y] = Particle::Water(dir);
                self.add_particle(x, y, z);
            }
        }
        self.update_ebo();
        self.update_vao();
    }

    pub fn add_wave_particles(&mut self, dir: Direction) {
        let y_range = 0..(WATER_GIRD_HEIGHT / 3 * 2) as usize;

        let (z_range, x_range) = match dir {
            Direction::South => ((WATER_GRID_WIDTH - 2 .. WATER_GRID_WIDTH - 1), (0..WATER_GRID_WIDTH - 1)),
            Direction::North => ((0..1), (0..WATER_GRID_WIDTH - 1)),
            Direction::East => ((0..WATER_GRID_WIDTH - 1), (WATER_GRID_WIDTH - 2 .. WATER_GRID_WIDTH - 1)),
            Direction::West => ((0..WATER_GRID_WIDTH - 1), (0..1)),
        };

        let dir = match dir {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        };

        for z in z_range.clone() {
            for x in x_range.clone() {
                for y in y_range.clone() {
                    if self.grid[z][x][y] == Particle::Empty {
                        self.grid[z][x][y] = Particle::Water(dir);
                        self.add_particle(x, y, z);
                    }
                }
            }
        }

        self.update_ebo();
        self.update_vao();
    }

    fn add_particle(&mut self, x: usize, y: usize, z: usize) {
        add_particle(&mut self.locations, &mut self.ib_data,
                     x, y, z,
                     WATER_GRID_WIDTH as u32, WATER_GIRD_HEIGHT as u32);
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

    pub fn dbg_print_col(&self, coord: (usize, usize)) {
        for elem in &self.grid[coord.0][coord.1] {
            print!("{:?} ", elem);
        }
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
    let step_h = 1. / (borders_h - 1) as f32;

    println!("Grid_heights rows: {}", grid_heights.len());
    println!("Grid_heights elems: {}", grid_heights[0].len());

    for (cur_row, nxt_row) in grid_heights.split_last().unwrap().1.iter().zip(grid_heights.split_first().unwrap().1) {
        let mut side: Vec<Vec<Particle>> = Vec::with_capacity(WATER_GIRD_HEIGHT - 1);

        for ((top_left, top_right), (bot_left, bot_right)) in
                cur_row.split_last().unwrap().1.iter()
                    .zip(cur_row.split_first().unwrap().1)
                    .zip(nxt_row.split_last().unwrap().1.iter()
                            .zip(nxt_row.split_first().unwrap().1))
        {
            let mut col: Vec<Particle> = Vec::with_capacity(borders_h);
            let cur_height = ((top_left + top_right + bot_right + bot_left) / 4. / step_h).ceil() as usize;
            let dir = get_direction(*top_left, *top_right, *bot_left, *bot_right);
            for _i in 0..cur_height {
                col.push(Particle::Border(dir));
            }
            for _i in cur_height..borders_h {
                col.push(Particle::Empty);
            }
            side.push(col);
        }
        borders.push(side);
    }
    borders
}

fn get_direction(top_left: f32, top_right: f32, bot_left: f32, bot_right: f32) -> Direction {
    let top = top_left + top_right;
    let bot = bot_left + bot_right;
    let left = top_left + bot_left;
    let right = top_right + bot_right;

    if (top > bot) && (top > left) && (top > right) {
        Direction::South
    }
    else if (bot > top) && (bot > left) && (bot > right) {
        Direction::North
    }
    else if (left > bot) && (left > top) && (left > right) {
        Direction::East
    }
    else {
        Direction::West
    }
}

fn generate_vertex_grid(grid_heights: &[Vec<f32>], borders_h: usize) -> Vec<Vertex> {
    let start = Utc::now();

    let mut vertices: Vec<Vertex> = Vec::with_capacity(WATER_GRID_WIDTH * WATER_GRID_WIDTH * WATER_GIRD_HEIGHT);
    let mut cur_coord = na::Vector3::new(-1., 0., -1.);
    let xz_step = 2. / (WATER_GRID_WIDTH - 1) as f32;
    let y_step = 1. / (WATER_GIRD_HEIGHT - 1) as f32;

    for row in grid_heights {
        cur_coord.x = -1.;
        for elem in row {
            cur_coord.y = 0.;
            for _i in 0..borders_h {
                vertices.push(cur_coord.into());
                cur_coord.y += y_step;
            }
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
