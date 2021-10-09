use failure::err_msg;
use gl_render::{ColorBuffer, Viewport};
use gl_render::uniform::HasUniform;
use resources::Resources;
use surface::Surface;
use crate::camera::MVP;
use controls::{Controls};
use grid::{Grid, GridingAlgo};
use water::{Water};

pub mod controls;
mod surface;
mod water;
mod grid;

pub struct GameData {
    gl: gl::Gl,
    viewport: Viewport,
    grid: Grid,
    surface: Surface,
    water: Water,
    mvp: MVP,
    color_buffer: ColorBuffer,
    pub controls: Controls,
    need_exit: bool,
}

pub const GRID_WIDTH: usize = 300;

impl GameData {
    pub fn new(gl: &gl::Gl, res: &Resources, grid_path: &str) -> Result<GameData, failure::Error> {
        let color_buffer: gl_render::ColorBuffer = (0.3, 0.3, 0.5).into(); // TODO add to config
        color_buffer.use_it(&gl);

        let viewport = gl_render::Viewport::for_window(900, 700); // TODO add size to config
        viewport.use_it(&gl);

        let grid = Grid::new(&res, grid_path, GRID_WIDTH, GridingAlgo::RadialBasisFunction)?;
        let surface = Surface::new(&res, &gl, grid.get_data())?;
        let water = Water::new(&res, &gl, grid.get_data())?;

        let mvp = MVP::new();
        surface.apply_uniform(&gl, &mvp, "mvp_transform").map_err(err_msg)?;
        water.apply_uniform(&gl, &mvp, "mvp_transform").map_err(err_msg)?;

        let controls = Controls::new();
        let need_exit = false;

        Ok(GameData { gl: gl.clone(), viewport, surface, mvp, color_buffer, controls, grid, water, need_exit })
    }

    pub fn resized(&mut self, w: i32, h: i32) -> Result<(), failure::Error> {
        self.viewport.update_size(w, h);
        self.viewport.use_it(&self.gl);
        self.mvp.projection_recalc(w, h);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }

    pub fn modulate(&mut self) -> Result<(), failure::Error> {
        if self.controls.is_rain {
            self.water.add_rain_particles();
        }
        self.water.modulate();
        self.apply_uniforms().map_err(err_msg)
    }

    pub fn render(&self) {
        self.color_buffer.clear(&self.gl);
        self.surface.render(&self.gl, gl::TRIANGLES); // TODO: add key for changing render mode
        self.water.render(&self.gl, gl::TRIANGLES);

        // TODO: depth buffer
        unsafe {
            self.gl.Clear(gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn need_exit(&self) -> bool {
        self.need_exit
    }

    fn apply_uniforms(&self) -> Result<(), failure::Error> {
        self.surface.apply_uniform(&self.gl, &self.mvp, "mvp_transform").map_err(err_msg)?;
        self.water.apply_uniform(&self.gl, &self.mvp, "mvp_transform").map_err(err_msg)?;
        Ok(())
    }

    pub fn init(&self) {
        unsafe {
            // TODO: depth buffer
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.DepthFunc(gl::LEQUAL);
            self.gl.DepthRange(0., 1.);
            self.gl.ClearDepth(1.);
        }
    }
}
