use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

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
    pub cam_capture:    KeyStatus,
    mouse_left_clk: na::Vector2<i32>,
    mouse_cur_pos: na::Vector2<i32>,
}

impl Controls {
    pub fn new() -> Controls {
        let mouse_left_clk = na::Vector2::new(0, 0);
        let mouse_cur_pos = na::Vector2::new(0, 0);
        Controls {
            exit:           KeyStatus::Released,
            flush:          KeyStatus::Released,
            add_water:      KeyStatus::Released,
            wave_s:         KeyStatus::Released,
            wave_n:         KeyStatus::Released,
            wave_e:         KeyStatus::Released,
            wave_w:         KeyStatus::Released,
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
            Keycode::Escape =>  self.exit       = status,
            Keycode::F =>       self.flush      = status,
            Keycode::W =>       self.add_water  = status,
            Keycode::Num8 =>    self.wave_n     = status,
            Keycode::Num2 =>    self.wave_s     = status,
            Keycode::Num4 =>    self.wave_w     = status,
            Keycode::Num6 =>    self.wave_e     = status,
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
            Actions::Flush    => self.flush      = KeyStatus::Released,
            Actions::AddWater => self.add_water  = KeyStatus::Released,
            Actions::WaveN    => self.wave_n     = KeyStatus::Released,
            Actions::WaveS    => self.wave_s     = KeyStatus::Released,
            Actions::WaveE    => self.wave_e     = KeyStatus::Released,
            Actions::WaveW    => self.wave_w     = KeyStatus::Released,
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
