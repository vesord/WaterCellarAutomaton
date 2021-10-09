use failure::err_msg;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use crate::game_data::{GameData, GRID_WIDTH};
use crate::game_data::grid::GridingAlgo;
use crate::game_data::water::Direction;

#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum KeyStatus {
    Pressed,
    Released,
}

impl From<KeyStatus> for bool {
    fn from(key_status: KeyStatus) -> Self {
        match key_status {
            KeyStatus::Pressed => true,
            KeyStatus::Released => false,
        }
    }
}

pub enum Actions {
    Flush,
    AddWater,
    WaveS,
    WaveN,
    WaveE,
    WaveW,
    Rain,
    Kriging,
    RadialBasis,
}

#[derive(Copy, Clone)]
pub struct Controls {
    pub exit:           KeyStatus,
    pub flush:          KeyStatus,
    pub add_water:      KeyStatus,
    pub wave_s:         KeyStatus,
    pub wave_n:         KeyStatus,
    pub wave_e:         KeyStatus,
    pub wave_w:         KeyStatus,
    pub rain:           KeyStatus,
    pub kriging:        KeyStatus,
    pub radial_basis:   KeyStatus,
    pub is_rain:        bool,
    pub cam_capture:    KeyStatus,
    mouse_left_clk: na::Vector2<i32>,
    mouse_cur_pos: na::Vector2<i32>,
}

impl Controls {
    pub fn new() -> Controls {
        let mouse_left_clk = na::Vector2::new(0, 0);
        let mouse_cur_pos = na::Vector2::new(0, 0);
        let is_rain = false;
        Controls {
            exit:           KeyStatus::Released,
            flush:          KeyStatus::Released,
            add_water:      KeyStatus::Released,
            wave_s:         KeyStatus::Released,
            wave_n:         KeyStatus::Released,
            wave_e:         KeyStatus::Released,
            wave_w:         KeyStatus::Released,
            kriging:        KeyStatus::Released,
            radial_basis:   KeyStatus::Released,
            rain:           KeyStatus::Released,
            is_rain,
            cam_capture:    KeyStatus::Released,
            mouse_left_clk,
            mouse_cur_pos,
        }
    }

    pub fn action_keyboard(&mut self, key: Option<Keycode>, status: KeyStatus) {
        let key = match key {
            None => return,
            Some(k) => k,
        };

        match key {
            Keycode::Escape =>  self.exit         = status,
            Keycode::F =>       self.flush        = status,
            Keycode::Q =>       self.add_water    = status,
            Keycode::W =>       self.wave_n       = status,
            Keycode::S =>       self.wave_s       = status,
            Keycode::A =>       self.wave_w       = status,
            Keycode::D =>       self.wave_e       = status,
            Keycode::R =>       self.rain         = status,
            Keycode::Num1 =>    self.radial_basis = status,
            Keycode::Num2 =>    self.kriging      = status,
            _ => (),
        }
    }

    pub fn action_mouse(&mut self, key: MouseButton, x: i32, y: i32, status: KeyStatus) {
        match key {
            MouseButton::Left => {
                self.cam_capture = status;
                if status == KeyStatus::Pressed {
                    self.mouse_left_clk.x = x;
                    self.mouse_left_clk.y = y;
                }
            },
            _ => (),
        }
    }

    pub fn action_mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_cur_pos.x = x;
        self.mouse_cur_pos.y = y;
    }

    pub fn reset_action(&mut self, action: Actions) {
        match action {
            Actions::Flush       => self.flush        = KeyStatus::Released,
            Actions::AddWater    => self.add_water    = KeyStatus::Released,
            Actions::WaveN       => self.wave_n       = KeyStatus::Released,
            Actions::WaveS       => self.wave_s       = KeyStatus::Released,
            Actions::WaveE       => self.wave_e       = KeyStatus::Released,
            Actions::WaveW       => self.wave_w       = KeyStatus::Released,
            Actions::Rain        => self.rain         = KeyStatus::Released,
            Actions::Kriging     => self.kriging      = KeyStatus::Released,
            Actions::RadialBasis => self.radial_basis = KeyStatus::Released,
        }
    }

    pub fn get_naviball(&self) -> na::Vector2<i32> {
        self.mouse_cur_pos - self.mouse_left_clk
    }

    pub fn save_mouse_clk_pos(&mut self) {
        self.mouse_left_clk.x = self.mouse_cur_pos.x;
        self.mouse_left_clk.y = self.mouse_cur_pos.y;
    }
}

impl GameData {
    pub fn process_input(&mut self) -> Result<(), failure::Error> {
        if self.controls.kriging.into() { self.action_set_kriging()? };
        if self.controls.radial_basis.into() { self.action_set_radial_basis()? };
        if self.controls.exit.into() { self.action_exit() };
        if self.controls.flush.into() { self.action_flush() };
        if self.controls.add_water.into() { self.action_add_water() };
        if self.controls.wave_n.into() { self.action_wave_n() };
        if self.controls.wave_s.into() { self.action_wave_s() };
        if self.controls.wave_w.into() { self.action_wave_w() };
        if self.controls.wave_e.into() { self.action_wave_e() };
        if self.controls.rain.into() { self.action_rain() };
        if self.controls.cam_capture.into() { self.action_cam_capture().map_err(err_msg)? };
        Ok(())
    }

    fn action_flush(&mut self) {
        println!("Flush!");
        self.controls.reset_action(Actions::Flush);
        self.water.flush();
    }

    fn action_add_water(&mut self) {
        println!("Add water");
        self.controls.reset_action(Actions::AddWater);
        self.water.increase_water_level();
    }

    fn action_wave_s(&mut self) {
        println!("Wave south");
        self.controls.reset_action(Actions::WaveS);
        self.water.add_wave_particles(Direction::South);
    }

    fn action_wave_n(&mut self) {
        println!("Wave north");
        self.controls.reset_action(Actions::WaveN);
        self.water.add_wave_particles(Direction::North);
    }

    fn action_wave_e(&mut self) {
        println!("Wave east");
        self.controls.reset_action(Actions::WaveE);
        self.water.add_wave_particles(Direction::East);
    }

    fn action_wave_w(&mut self) {
        println!("Wave west");
        self.controls.reset_action(Actions::WaveW);
        self.water.add_wave_particles(Direction::West);
    }

    fn action_rain(&mut self) {
        self.controls.reset_action(Actions::Rain);
        self.controls.is_rain = !self.controls.is_rain;
        match self.controls.is_rain {
            true => println!("Rain start"),
            false => println!("Rain stop"),
        }
    }

    fn action_set_kriging(&mut self) -> Result<(), failure::Error> {
        println!("Griding algorithm: Kriging");
        self.controls.reset_action(Actions::Kriging);
        self.action_flush();
        self.grid.update_grid(GRID_WIDTH, GridingAlgo::Kriging);
        self.water.set_grid(&self.grid.get_data());
        self.surface.set_grid(&self.grid.get_data())?;
        Ok(())
    }

    fn action_set_radial_basis(&mut self) -> Result<(), failure::Error> {
        println!("Griding algorithm: Radial basis function");
        self.controls.reset_action(Actions::RadialBasis);
        self.action_flush();
        self.grid.update_grid(GRID_WIDTH, GridingAlgo::RadialBasisFunction);
        self.water.set_grid(&self.grid.get_data());
        self.surface.set_grid(&self.grid.get_data())?;
        Ok(())
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

    fn action_exit(&mut self) {
        self.need_exit = true;
    }
}
