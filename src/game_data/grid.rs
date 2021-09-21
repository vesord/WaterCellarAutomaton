use resources::Resources;
use failure::err_msg;
use std::ffi::CString;
use crate::types::Point3;

pub struct Grid {
    data: Vec<Vec<f32>>,
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
    pub fn new(res: &Resources, grid_path: &str, size: usize) -> Result<Grid, failure::Error> {
        let mut empty_grid: Vec<Vec<f32>> = vec![vec![0.; size as usize]; size as usize];
        let input_array = Grid::get_user_grid(res, grid_path)?;

        let input_array_zero_edges = Grid::add_zero_edge_to_user_grid(&input_array, size);

        let grid = Grid::radial_basis_function_griding(size, &input_array);

        //
        // // TODO: remove this
        // empty_grid[49][49] = 0.7;

        Ok(Grid {
            data: grid,
        })
    }

    pub fn get_data(&self) -> &Vec<Vec<f32>> {
        &self.data
    }

    fn get_user_grid(res: &Resources, grid_path: &str) -> Result<Vec<Point3>, failure::Error> {
        let grid_file = res.load_cstring(grid_path).map_err(err_msg)?;
        let grid_str = grid_str2file(grid_file, grid_path)?;
        // println!("grid str: {:?}", grid_str);
        let grid_lines: Vec<&str> = grid_str.split("\n").collect();
        // println!("grid lines: {:?}", grid_lines);
        let grid_points_str = grid_lines2points_str(&grid_lines)?;
        // println!("grid points str: {:?}", grid_points_str);
        let grid_points_f32 = grid_points_str2points_f32(&grid_points_str)?;
        // println!("grid points f32: {:?}", grid_points_f32);
        let grid: Vec<Point3> = grid_points_f32to_grid(&grid_points_f32)?;
        println!("grid: {:?}", grid);
        Ok(grid)
    }

    fn add_zero_edge_to_user_grid(points: &Vec<Point3>, size: usize) -> Vec<Point3> {
        let mut grid_zeroed: Vec<Point3> = Vec::with_capacity(points.len() + (size - 1) * 4);
        let step = 2. / size as f32;
        let mut coord = -1. - step;
        for i in 0..size {
            coord += step;
            grid_zeroed.push((coord, 0., -1.).into());
            grid_zeroed.push((-1., 0., coord).into());
            grid_zeroed.push((coord, 0., 1.).into());
            grid_zeroed.push((1., 0., coord).into());
        }
        for user_point in points {
            grid_zeroed.push(*user_point);
        }
        grid_zeroed
    }

    // Makes isomorphic 2d grid on [-1;1] through input points. Edges are zeroed.
    fn radial_basis_function_griding(size: usize, poles: &Vec<Point3>) -> Vec<Vec<f32>> {
        let step: f32 = 2. / size as f32;
        let mut cur_point: Point3 = (-1. - step, 0., -1. - step).into(); // (x, -z)
        let mut grid: Vec<Vec<f32>> = vec![vec![0.; size]; size];

        for mut row in &mut grid {
            cur_point.z += step;
            for mut elem in row {
                cur_point.x += step;
                *elem = Grid::rbf_calculate_point(&cur_point, poles);
                println!("Elem: {} ", *elem);
            }
            cur_point.x = -1. - step;
        }
        grid
    }

    fn rbf_calculate_point(cur_point: &Point3, poles: &Vec<Point3>) -> f32 {
        let mut distances: Vec<f32> = Vec::with_capacity(poles.len());
        for pole in poles {
            let dist = max(cur_point.distance_xz(pole), f32::MIN_POSITIVE * 100.);
            // println!("CP: {}, Pole: {}, Rev dist: {}", cur_point, pole, rev_dist);
            distances.push(dist);
        }
        let weights: Vec<f32> = distances.iter().map(|dist| -5. * (dist * dist) + 1.).collect(); // TODO: add function to config
        let mut y_value = 0.;
        for (w, d) in weights.iter().zip(poles) {
            let weight = max(*w, 0.);
            y_value = max(d.y * weight, y_value);
            println!("y: {}, w: {}", d.y, weight);

        }
        y_value
    }

    fn krigging_calculate_point(cur_point: &Point3, poles: &Vec<Point3>) -> f32 {
        let mut rev_distances: Vec<f32> = Vec::with_capacity(poles.len());
        let mut sum_rev_dists: f32 = 0.;
        for pole in poles {
            let rev_dist = 1. / max(cur_point.distance_xz(pole), f32::MIN_POSITIVE * 100.);
            // println!("CP: {}, Pole: {}, Rev dist: {}", cur_point, pole, rev_dist);
            sum_rev_dists += rev_dist;
            rev_distances.push(rev_dist);
        }
        // println!("SUM REV DIST: {}", sum_rev_dists);
        let weights: Vec<f32> = rev_distances.iter().map(|rev_dist| rev_dist / sum_rev_dists).collect();
        let mut y_value = 0.;
        for (w, d) in weights.iter().zip(poles) {
            // println!("y: {}, w: {}", d.y, w);
            y_value += d.y * w;
        }
        y_value
    }
}

fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    }
    else {
        b
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

fn grid_points_f32to_grid(points: &Vec<Vec<f32>>) -> Result<Vec<Point3>, Error> {
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
        Ok((x, y, z).into())
    }).collect::<Result<Vec<Point3>, Error>>()
}
