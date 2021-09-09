#[macro_use] extern crate failure;
#[macro_use] extern crate render_gl_derive;
extern crate sdl2;
extern crate gl_builder as gl;
extern crate resources;
extern crate gl_render;

use std::path::Path;
use failure::err_msg;
use sdl2::event::Event;

mod debug;
mod triangle;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 6);

    let window = video_subsystem
        .window("HumanGL", 900, 700)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let mut viewport = gl_render::Viewport::for_window(900, 700);
    viewport.use_it(&gl);

    let color_buffer: gl_render::ColorBuffer = (0.3, 0.3, 0.5).into();
    color_buffer.use_it(&gl);

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    let res = resources::Resources::from_relative_exe_path(Path::new("shaders"))?;

    let triangle = triangle::Triangle::new(&res, &gl)?;

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.use_it(&gl);
                }
                _ => {},
            }
        }

        color_buffer.clear(&gl);
        triangle.render(&gl);

        window.gl_swap_window();
    }

    Ok(())
}
