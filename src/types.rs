use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Vec3f32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Simple point, but y-component is up direction
pub type Point3 = Vec3f32;

impl From<(f32, f32, f32)> for Point3 {
    fn from(other: (f32, f32, f32)) -> Self {
        Point3 {
            x: other.0,
            y: other.1,
            z: other.2,
        }
    }
}

impl Point3 {
    pub fn distance_xz(&self, to: &Point3) -> f32 {
        ((self.x - to.x) * (self.x - to.x) + (self.z - to.z) * (self.z - to.z)).sqrt()
    }
}

impl Display for Point3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

