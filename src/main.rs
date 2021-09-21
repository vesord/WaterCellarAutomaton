#[macro_use] extern crate failure;
#[macro_use] extern crate render_gl_derive;
extern crate sdl2;
extern crate gl_builder as gl;
extern crate resources;
extern crate gl_render;
extern crate nalgebra as na;

use std::path::Path;
use failure::err_msg;
use sdl2::event::{Event, WindowEvent};
use game_data::{controls::KeyStatus, GameData};
use crate::initialization::{create_window, set_gl_attr};
use std::env;

mod debug;
mod initialization;
mod camera;
mod game_data;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();
    let grid_path = match args.len() {
        1 => "grids/grid.mod1".to_owned(),
        2 => "grids/".to_owned() + &args[1],
        _ => { println!("Too much arguments"); return; }
    };

    if let Err(e) = run(&grid_path) {
        println!("{}", debug::failure_to_string(e));
    }
}

fn run(grid_path: &str) -> Result<(), failure::Error> {
    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;
    set_gl_attr(&video_subsystem);
    let window = create_window(&video_subsystem).map_err(err_msg)?;
    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    let res = resources::Resources::from_relative_exe_path(Path::new("assets"))?;

    let mut gd = GameData::new(&gl, &res, grid_path).map_err(err_msg)?;
    gd.init();

    gd.set_grid()?;


    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main,
                Event::Window { win_event: WindowEvent::Resized(w, h), .. } =>
                    gd.resized(w, h).map_err(err_msg)?,
                Event::KeyUp {keycode, ..} => gd.controls.action_keyboard(keycode, KeyStatus::Released),
                Event::KeyDown {keycode, ..} => gd.controls.action_keyboard(keycode, KeyStatus::Pressed),
                Event::MouseButtonUp {mouse_btn, x, y, ..} => gd.controls.action_mouse(mouse_btn, x, y, KeyStatus::Released),
                Event::MouseButtonDown {mouse_btn, x, y, ..} => gd.controls.action_mouse(mouse_btn, x, y,KeyStatus::Pressed),
                Event::MouseMotion {x, y, ..} => gd.controls.action_mouse_move(x, y),
                Event::MouseWheel {y, ..} => gd.controls.action_mouse_wheel(y),
                _ => {},
            }
        }
        gd.process_input()?;
        gd.modulate();
        gd.render();
        window.gl_swap_window();
    }
    Ok(())
}
