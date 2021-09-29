use failure::err_msg;
use gl_render::{ColorBuffer, Viewport};
use gl_render::uniform::HasUniform;
use resources::Resources;
use surface::Surface;
use crate::camera::MVP;
use controls::{Actions, Controls};
use crate::game_data::grid::{Grid, GridingAlgo};
use crate::game_data::water::{Water, WaveDirection};
use std::time::Duration;

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
}

pub const GRID_WIDTH: usize = 200;

impl GameData {
    pub fn new(gl: &gl::Gl, res: &Resources, grid_path: &str) -> Result<GameData, failure::Error> {
        let color_buffer: gl_render::ColorBuffer = (0.3, 0.3, 0.5).into(); // TODO add to config
        color_buffer.use_it(&gl);

        let viewport = gl_render::Viewport::for_window(900, 700); // TODO add size to config
        viewport.use_it(&gl);

        let grid = Grid::new(&res, grid_path, GRID_WIDTH, GridingAlgo::RadialBasisFunction)?;  // TODO: add size to config (200 in subj)

        let mut surface = Surface::new(&res, &gl)?;
        surface.set_grid(grid.get_data())?;                     // TODO: move to ::new()

        let mut water = Water::new(&res, &gl)?;
        water.set_grid(grid.get_data());    // TODO: move to ::new()

        let mvp = MVP::new();
        surface.apply_uniform(&gl, &mvp, "mvp_transform").map_err(err_msg)?;
        water.apply_uniform(&gl, &mvp, "mvp_transform").map_err(err_msg)?;

        let controls = Controls::new();

        Ok(GameData { gl: gl.clone(), viewport, surface, grid, mvp, color_buffer, controls, water })
    }

    pub fn resized(&mut self, w: i32, h: i32) -> Result<(), failure::Error> {
        self.viewport.update_size(w, h);
        self.viewport.use_it(&self.gl);
        self.mvp.projection_recalc(w, h);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }

    pub fn modulate(&mut self) -> Result<(), failure::Error> {
        // self.mvp.model_rotate_y(3.14 / 360.);
        // self.water.loop_add_water();
        // self.water.add_rain_particles();
        // self.water.modulate();
        self.apply_uniforms().map_err(err_msg)
    }

    pub fn process_input(&mut self) -> Result<(), failure::Error> {
        if self.controls.flush.into() { self.action_flush() };
        if self.controls.add_water.into() { self.action_add_water() };        // TODO: rename inc_water / dec_water
        if self.controls.wave_n.into() { self.action_wave_n() };
        if self.controls.wave_s.into() { self.action_wave_s() };
        if self.controls.wave_w.into() { self.action_wave_w() };
        if self.controls.wave_e.into() { self.action_wave_e() };
        if self.controls.cam_capture.into() { self.action_cam_capture().map_err(err_msg)? };
        if self.controls.zoom != 0 { self.action_zoom().map_err(err_msg)? };

        Ok(())
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

    fn apply_uniforms(&self) -> Result<(), failure::Error> {
        self.surface.apply_uniform(&self.gl, &self.mvp, "mvp_transform").map_err(err_msg)?;
        self.water.apply_uniform(&self.gl, &self.mvp, "mvp_transform").map_err(err_msg)?;
        Ok(())
    }

    pub fn init(&self) {
        unsafe {
            // gl.Disable(gl::CULL_FACE);
            // gl.FrontFace(gl::CCW);

            // TODO: depth buffer
            self.gl.Enable(gl::DEPTH_TEST);
            self.gl.DepthFunc(gl::LEQUAL);
            self.gl.DepthRange(0., 1.);
            self.gl.ClearDepth(1.);
        }
    }
    // TODO: remove if not need
    // pub fn set_grid(&mut self) -> Result<(), failure::Error> {
    //     self.surface.set_grid(self.grid.get_data())
    // }

    fn action_flush(&mut self) {
        println!("FLUSH!");
        self.controls.reset_action(Actions::Flush);
        self.water.flush();
    }

    fn action_add_water(&mut self) {
        println!("ADD WATER");
        self.controls.reset_action(Actions::AddWater);
        self.water.increase_water_level();
    }

    fn action_wave_s(&mut self) {
        println!("WAVE SOUTH");
        self.controls.reset_action(Actions::WaveS);
        self.water.add_wave_particles(WaveDirection::South);
        // self.water.dbg_move_south();
    }

    fn action_wave_n(&mut self) {
        println!("WAVE NORTH");
        self.controls.reset_action(Actions::WaveN);
        self.water.add_wave_particles(WaveDirection::North);
        // self.water.dbg_move_north();
    }

    fn action_wave_e(&mut self) {
        println!("WAVE EAST");
        self.controls.reset_action(Actions::WaveE);
        self.water.add_wave_particles(WaveDirection::East);
        // self.water.dbg_move_east();
    }

    fn action_wave_w(&mut self) {
        println!("WAVE WEST");
        self.controls.reset_action(Actions::WaveW);
        self.water.add_wave_particles(WaveDirection::West);
        // self.water.dbg_move_west();
    }

    fn action_cam_capture(&mut self) -> Result<(), failure::Error> {
        let naviball: na::Vector2<i32> = self.controls.get_naviball();
        self.controls.save_mouse_clk_pos();
        if naviball.x == 0 && naviball.y == 0 {
            return Ok(())
        }

        let naviball: na::Vector2<f32> = na::Vector2::new(
            (naviball.x) as f32 / (self.viewport.w) as f32,
            (naviball.y) as f32 / (self.viewport.h) as f32 );

        self.mvp.view_rotate_naviball(naviball);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }

    fn action_zoom(&mut self) -> Result<(), failure::Error> {
        let zoom: f32 = self.controls.zoom as f32 / 3.;
        self.controls.reset_action(Actions::Zoom);
        self.mvp.view_translate_zoom(zoom);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }
}
