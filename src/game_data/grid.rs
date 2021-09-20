use resources::Resources;
use failure::err_msg;
use std::ffi::CString;

pub struct Grid {
    data: Vec<Vec<f32>>,
}

impl From<Vec<f32>> for Grid {
    fn from(points: Vec<f32>) -> Self {
        todo!()
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Unable to convert file {} to string", name)]
    UnableConvertFileToString { name: String },
    #[fail(display = "Point {} does not have 3 components (x, y, z)", name)]
    PointDoNotHave3Components { name: String },
    #[fail(display = "Non f32 component found: {}", name)]
    ComponentIsNotF32 { name: String },
    #[fail(display = "Component X is not in range [-1;1]: {}", name)]
    ComponentXNotValid { name: String },
    #[fail(display = "Component Y is not in range [0;1]: {}", name)]
    ComponentYNotValid { name: String },
    #[fail(display = "Component Z is not in range [-1;1]: {}", name)]
    ComponentZNotValid { name: String },
}

impl Grid {
    pub fn new(res: &Resources, grid_path: &str, size: u32) -> Result<Grid, failure::Error> {
        let mut empty_grid: Vec<Vec<f32>> = vec![vec![0.; size as usize]; size as usize];
        let input_array = Grid::from_res(res, grid_path)?;

        // TODO: remove this
        empty_grid[49][49] = 0.7;

        Ok(Grid {
            data: empty_grid,
        })
    }

    pub fn get_data(&self) -> &Vec<Vec<f32>> {
        &self.data
    }

    fn from_res(res: &Resources, grid_path: &str) -> Result<Vec<(f32, f32, f32)>, failure::Error> {
        let grid_file = res.load_cstring(grid_path).map_err(err_msg)?;
        let grid_str = grid_str2file(grid_file, grid_path)?;
        // println!("grid str: {:?}", grid_str);
        let grid_lines: Vec<&str> = grid_str.split("\n").collect();
        // println!("grid lines: {:?}", grid_lines);
        let grid_points_str = grid_lines2points_str(&grid_lines)?;
        // println!("grid points str: {:?}", grid_points_str);
        let grid_points_f32 = grid_points_str2points_f32(&grid_points_str)?;
        // println!("grid points f32: {:?}", grid_points_f32);
        let grid = grid_points_f32to_grid(&grid_points_f32)?;
        println!("grid: {:?}", grid);
        Ok(grid)
    }
}

fn grid_str2file(str: CString, filename: &str) -> Result<String, Error> {
    str.into_string().map_err(
        |e| Error::UnableConvertFileToString { name: filename.into() }
    )
}

fn grid_lines2points_str<'a>(lines: &Vec<&'a str>) -> Result<Vec<Vec<&'a str>>, Error> {
    lines.iter().map(|line| {
        let coords: Vec<&str> = line.split(",").collect();
        match coords.len() {
            3 => Ok(coords),
            _ => Err(Error::PointDoNotHave3Components {name: line.to_string()})
        }}
    ).collect::<Result<Vec<Vec<&str>>, Error>>()
}

fn grid_points_str2points_f32(points: &Vec<Vec<&str>>) -> Result<Vec<Vec<f32>>, Error> {
    points.iter().map(|coords| {
        coords.iter().map(|coord| {
            coord.parse::<f32>().map_err(|e| Error::ComponentIsNotF32 { name: coord.to_string() })
        }).collect::<Result<Vec<f32>, Error>>()
    }).collect::<Result<Vec<Vec<f32>>, Error>>()
}

fn grid_points_f32to_grid(points: &Vec<Vec<f32>>) -> Result<Vec<(f32, f32, f32)>, Error> {
    points.iter().map(|point| {
        let x = match point[0] {
            x if x >= -1. && x <= 1. => Ok(x),
            _ => Err(Error::ComponentXNotValid { name: point[0].to_string() }),
        }?;
        let y = match point[1] {
            y if y >= 0. && y <= 1. => Ok(y),
            _ => Err(Error::ComponentYNotValid { name: point[1].to_string() }),
        }?;
        let z = match point[2] {
            z if z >= -1. && z <= 1. => Ok(z),
            _ => Err(Error::ComponentZNotValid { name: point[2].to_string() }),
        }?;
        Ok((x, y, z))
    }).collect::<Result<Vec<(f32, f32, f32)>, Error>>()
}
