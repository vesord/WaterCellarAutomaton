use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;

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
    CamCapture,
    Zoom,
}

#[derive(Copy, Clone)]
pub struct Controls {
    pub flush:          KeyStatus,
    pub add_water:      KeyStatus,
    pub wave_s:         KeyStatus,
    pub wave_n:         KeyStatus,
    pub wave_e:         KeyStatus,
    pub wave_w:         KeyStatus,
    pub cam_capture:    KeyStatus,
    pub zoom:           i32,
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            flush:          KeyStatus::Released,
            add_water:      KeyStatus::Released,
            wave_s:         KeyStatus::Released,
            wave_n:         KeyStatus::Released,
            wave_e:         KeyStatus::Released,
            wave_w:         KeyStatus::Released,
            cam_capture:    KeyStatus::Released,
            zoom:           0,
        }
    }

    pub fn action_keyboard(&mut self, key: Option<Keycode>, status: KeyStatus) {
        let key = match key {
            None => return,
            Some(k) => k,
        };

        match key {
            Keycode::F =>       self.flush      = status,
            Keycode::L =>       self.add_water  = status,
            Keycode::Num8 =>    self.wave_n     = status,
            Keycode::Num2 =>    self.wave_s     = status,
            Keycode::Num4 =>    self.wave_w     = status,
            Keycode::Num6 =>    self.wave_e     = status,
            _ => (),
        }
    }

    pub fn action_mouse(&mut self, key: MouseButton, status: KeyStatus) {
        match key {
            MouseButton::Left => self.cam_capture = status,
            _ => (),
        }
    }

    pub fn action_mouse_wheel(&mut self, value: i32) {
        self.zoom += value
    }

    pub fn reset_action(&mut self, action: Actions) {
        match action {
            Actions::Flush    => self.flush      = KeyStatus::Released,
            Actions::AddWater => self.add_water  = KeyStatus::Released,
            Actions::WaveN    => self.wave_n     = KeyStatus::Released,
            Actions::WaveS    => self.wave_s     = KeyStatus::Released,
            Actions::WaveE    => self.wave_e     = KeyStatus::Released,
            Actions::WaveW    => self.wave_w     = KeyStatus::Released,
            Actions::CamCapture => self.cam_capture = KeyStatus::Released,
            Actions::Zoom => self.zoom = 0,
        }
    }
}
