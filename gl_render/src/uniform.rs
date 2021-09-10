pub trait HasUniform<T> {
    fn apply_uniform(&self, gl: &gl::Gl, data: &T, name: &str) -> Result<(), failure::Error>;
}