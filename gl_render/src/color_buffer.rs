use gl;

pub struct ColorBuffer {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<(f32, f32, f32, f32)> for ColorBuffer {
    fn from(other: (f32, f32, f32, f32)) -> Self {
        ColorBuffer::from_color(other.0, other.1, other.2, other.3)
    }
}

impl From<(f32, f32, f32)> for ColorBuffer {
    fn from(other: (f32, f32, f32)) -> Self {
        ColorBuffer::from_color(other.0, other.1, other.2, 1.)
    }
}

impl ColorBuffer {
    // pub fn from_color_buffer(other: ColorBuffer) -> ColorBuffer {
    //     ColorBuffer {
    //         r: other.r, g: other.g, b: other.b, a: other.a,
    //     }
    // }

    fn from_color(r: f32, g: f32, b: f32, a: f32) -> ColorBuffer {
        ColorBuffer {
            r, g, b, a
        }
    }

    // pub fn update_color(&mut self, color: ColorBuffer) {
    //     self.r = color.r;
    //     self.g = color.g;
    //     self.b = color.b;
    //     self.a = color.a;
    // }

    pub fn use_it(&self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(self.r, self.g, self.b, self.a);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}