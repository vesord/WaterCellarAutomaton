use crate::game_data::water::{WATER_GIRD_HEIGHT, WATER_GRID_WIDTH};

pub const POINTS_PER_PARTICLE: usize = 6;

#[repr(C, packed)]
pub struct ParticleShape {
    t0: TriangleIdx,
    t1: TriangleIdx,
}

impl ParticleShape {
    pub fn new(x: u32, y: u32, z: u32, xz_size: u32, y_size: u32) -> ParticleShape {
        let p0 = z * xz_size * y_size + x * y_size + y; // Top left
        let p1 = p0 + y_size;                              // Top right
        let p2 = p0 + y_size * (xz_size + 1);              // Bot right
        let p3 = p0 + y_size * xz_size;                  // Bot left

        ParticleShape {
            t0: (p0, p1, p2).into(),
            t1: (p0, p3, p2).into(),
        }
    }

    pub fn move_down(&mut self) {
        self.t0.move_down();
        self.t1.move_down();
    }

    pub fn move_north(&mut self) {
        self.t0.move_north();
        self.t1.move_north();
    }

    pub fn move_south(&mut self) {
        self.t0.move_south();
        self.t1.move_south();
    }

    pub fn move_west(&mut self) {
        self.t0.move_west();
        self.t1.move_west();
    }

    pub fn move_east(&mut self) {
        self.t0.move_east();
        self.t1.move_east();
    }
}

#[repr(C, packed)]
struct TriangleIdx {
    i0: u32,
    i1: u32,
    i2: u32,
}

impl From<(u32, u32, u32)> for TriangleIdx {
    fn from(other: (u32, u32, u32)) -> Self {
        TriangleIdx {
            i0: other.0, i1: other.1, i2: other.2,
        }
    }
}

impl TriangleIdx {
    pub fn move_down(&mut self) {
        self.i0 -= 1;
        self.i1 -= 1;
        self.i2 -= 1;
    }

    pub fn move_north(&mut self) {
        self.i0 -= (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
        self.i1 -= (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
        self.i2 -= (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
    }

    pub fn move_south(&mut self) {
        self.i0 += (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
        self.i1 += (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
        self.i2 += (WATER_GRID_WIDTH * WATER_GIRD_HEIGHT) as u32;
    }

    pub fn move_west(&mut self) {
        self.i0 -= (WATER_GIRD_HEIGHT) as u32;
        self.i1 -= (WATER_GIRD_HEIGHT) as u32;
        self.i2 -= (WATER_GIRD_HEIGHT) as u32;
    }

    pub fn move_east(&mut self) {
        self.i0 += (WATER_GIRD_HEIGHT) as u32;
        self.i1 += (WATER_GIRD_HEIGHT) as u32;
        self.i2 += (WATER_GIRD_HEIGHT) as u32;
    }
}

