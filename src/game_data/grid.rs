pub struct Grid {
    data: Vec<Vec<f32>>,
}

impl Grid {
    pub fn new(size: u32) -> Grid {
        let mut empty_grid: Vec<Vec<f32>> = vec![vec![0.; size as usize]; size as usize];

        empty_grid[49][49] = 0.7;

        Grid {
            data: empty_grid,
        }
    }

    pub fn get_data(&self) -> &Vec<Vec<f32>> {
        &self.data
    }
}