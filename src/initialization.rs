use sdl2::VideoSubsystem;
use sdl2::video::Window;
use failure::err_msg;

pub fn set_gl_attr(video: &VideoSubsystem) {
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 6);
}

pub fn create_window(video: &VideoSubsystem) -> Result<Window, failure::Error> {
    let window = video
        .window("mod1", 900, 700)
        .opengl()
        .resizable()
        .build().map_err(err_msg)?;
    Ok(window)
}