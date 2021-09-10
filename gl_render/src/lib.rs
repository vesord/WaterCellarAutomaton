#[macro_use] extern crate failure;
extern crate resources;
extern crate gl_builder as gl;

mod shader;
pub use self::shader::{Shader, Program, Error};

mod viewport;
pub use self::viewport::Viewport;

mod color_buffer;
pub use self::color_buffer::ColorBuffer;

pub mod data;
pub mod buffer;

pub mod uniform;
