#[macro_use] extern crate failure;
#[macro_use] extern crate render_gl_derive;
extern crate sdl2;
extern crate gl_builder as gl;
extern crate resources;
extern crate gl_render;
extern crate nalgebra as na;

use std::path::Path;
use failure::err_msg;
use sdl2::event::{WindowEvent, Event};
use crate::initialization::{set_gl_attr, create_window};
use game_data::{GameData, controls::KeyStatus};

mod debug;
mod initialization;
mod surface;
mod camera;
mod game_data;

fn main() {
    if let Err(e) = run() {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;
    set_gl_attr(&video_subsystem);
    let window = create_window(&video_subsystem).map_err(err_msg)?;
    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    let res = resources::Resources::from_relative_exe_path(Path::new("shaders"))?;

    let mut gd = GameData::new(&gl, &res).map_err(err_msg)?;
    gd.init();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::Window { win_event: WindowEvent::Resized(w, h), .. } =>
                    gd.resized(w, h).map_err(err_msg)?,
                Event::KeyUp {keycode, ..} => gd.controls.action_keyboard(keycode, KeyStatus::Released),
                Event::KeyDown {keycode, ..} => gd.controls.action_keyboard(keycode, KeyStatus::Pressed),
                Event::MouseButtonUp {mouse_btn, ..} => gd.controls.action_mouse(mouse_btn, KeyStatus::Released),
                Event::MouseButtonDown {mouse_btn, ..} => gd.controls.action_mouse(mouse_btn, KeyStatus::Pressed),
                Event::MouseWheel {y, ..} => gd.controls.action_mouse_wheel(y),
                _ => {},
            }
        }
        gd.process_input();
        gd.render();
        window.gl_swap_window();
    }
    Ok(())
}
