use gl_render::{Viewport, ColorBuffer};
use crate::surface::Surface;
use crate::camera::MVP;
use resources::Resources;
use failure::err_msg;
use gl_render::uniform::HasUniform;
use crate::game_data::controls::{Controls, Actions};

pub mod controls;

pub struct GameData {
    gl: gl::Gl,
    viewport: Viewport,
    surface: Surface,
    mvp: MVP,
    color_buffer: ColorBuffer,
    pub controls: Controls,
}

impl GameData {
    pub fn new(gl: &gl::Gl, res: &Resources) -> Result<GameData, failure::Error> {
        let color_buffer: gl_render::ColorBuffer = (0.3, 0.3, 0.5).into();
        color_buffer.use_it(&gl);

        let viewport = gl_render::Viewport::for_window(900, 700);
        viewport.use_it(&gl);

        let surface = Surface::new(&res, &gl)?;

        let mvp = MVP::new();
        surface.apply_uniform(&gl, &mvp, "mvp_transform").map_err(err_msg)?;

        let controls = Controls::new();

        Ok(GameData { gl: gl.clone(), viewport, surface, mvp, color_buffer, controls })
    }

    pub fn resized(&mut self, w: i32, h: i32) -> Result<(), failure::Error> {
        self.viewport.update_size(w, h);
        self.viewport.use_it(&self.gl);
        self.mvp.recalc_projection(w, h);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }

    pub fn process_input(&mut self) -> Result<(), failure::Error> {
        if self.controls.flush.into() { self.action_flush() };
        if self.controls.add_water.into() { self.action_add_water() };
        if self.controls.wave_n.into() { self.action_wave_n() };
        if self.controls.wave_s.into() { self.action_wave_s() };
        if self.controls.wave_w.into() { self.action_wave_w() };
        if self.controls.wave_e.into() { self.action_wave_e() };
        if self.controls.cam_capture.into() { self.action_cam_capture().map_err(err_msg)? };

        Ok(())
    }

    pub fn render(&self) {
        self.color_buffer.clear(&self.gl);
        self.surface.render(&self.gl);
    }

    fn apply_uniforms(&self) -> Result<(), failure::Error> {
        self.surface.apply_uniform(&self.gl, &self.mvp, "mvp_transform").map_err(err_msg)?;
        Ok(())
    }

    pub fn init(&self) {
        unsafe {
            // gl.Disable(gl::CULL_FACE);
            // gl.FrontFace(gl::CCW);
            // gl.Enable(gl::DEPTH_TEST);
            // gl.DepthFunc(gl::LEQUAL);
            // gl.DepthRange(0., 1.);
            // gl.ClearDepth(1.);
        }
    }

    fn action_flush(&mut self) {
        self.controls.reset_action(Actions::Flush);
        println!("FLUSH!")
    }

    fn action_add_water(&mut self) {
        self.controls.reset_action(Actions::AddWater);
        println!("ADD WATER")
    }

    fn action_wave_s(&mut self) {
        self.controls.reset_action(Actions::WaveS);
        println!("WAVE SOUTH")
    }

    fn action_wave_n(&mut self) {
        self.controls.reset_action(Actions::WaveN);
        println!("WAVE NORTH")
    }

    fn action_wave_e(&mut self) {
        self.controls.reset_action(Actions::WaveE);
        println!("WAVE EAST")
    }

    fn action_wave_w(&mut self) {
        self.controls.reset_action(Actions::WaveW);
        println!("WAVE WEST")
    }

    fn action_cam_capture(&mut self) -> Result<(), failure::Error> {
        let naviball: na::Vector2<i32> = self.controls.get_naviball();
        self.controls.save_mouse_clk_pos();
        if naviball.x == 0 && naviball.y == 0 {
            return Ok(())
        }

        let naviball: na::Vector2<f32> = na::Vector2::new(
            (naviball.x) as f32 / (self.viewport.w) as f32,
            -(naviball.y) as f32 / (self.viewport.h) as f32 );

        self.mvp.recalc_model_naviball(naviball);
        self.apply_uniforms().map_err(err_msg)?;
        Ok(())
    }
}