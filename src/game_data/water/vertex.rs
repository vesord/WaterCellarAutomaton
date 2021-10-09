use gl_render::{data};

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
}

impl From<(f32, f32, f32)> for Vertex {
    fn from(elem: (f32, f32, f32)) -> Self {
        Vertex { pos: elem.into() }
    }
}

impl From<na::Vector3<f32>> for Vertex {
    fn from(v: na::Vector3<f32>) -> Self {
        Vertex { pos: (v.x, v.y, v.z).into() }
    }
}