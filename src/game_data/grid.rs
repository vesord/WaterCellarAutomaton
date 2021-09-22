use resources::Resources;
use failure::err_msg;
use std::ffi::CString;

pub struct Grid {
    poles: Vec<na::Vector3<f32>>,
    data: Vec<Vec<f32>>,
}

pub enum GridingAlgo {
    RadialBasisFunction,
    Kriging,
}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Unable to convert file {} to string", name)]
    UnableConvertFileToString { name: String },
    #[fail(display = "Point {} does not have 3 components (x, y, z)", name)]
    PointDoNotHave3Components { name: String },
    #[fail(display = "Non f32 component found: {}, {}", name, message)]
    ComponentIsNotF32 { name: String, message: String },
    #[fail(display = "Component X is not in range [-1;1]: {}", name)]
    ComponentXNotValid { name: String },
    #[fail(display = "Component Y is not in range [0;1]: {}", name)]
    ComponentYNotValid { name: String },
    #[fail(display = "Component Z is not in range [-1;1]: {}", name)]
    ComponentZNotValid { name: String },
}

impl Grid {
    pub fn new(res: &Resources, grid_path: &str, size: usize, _griding_algo: GridingAlgo) -> Result<Grid, failure::Error> {
        let input_array = Grid::get_user_grid(res, grid_path)?;
        let input_array = Grid::add_zeros_to_edges(&input_array, 10);    // TODO: config
        let grid = Grid::make_grid(size, &input_array, GridingAlgo::RadialBasisFunction);   // TODO: pass griging_algo
        Ok(Grid {
            poles: input_array,
            data: grid,
        })
    }

    pub fn get_data(&self) -> &Vec<Vec<f32>> {
        &self.data
    }

    fn get_user_grid(res: &Resources, grid_path: &str) -> Result<Vec<na::Vector3<f32>>, failure::Error> {
        let grid_file = res.load_cstring(grid_path).map_err(err_msg)?;
        let grid_str = grid_str2file(grid_file, grid_path)?;
        // println!("grid str: {:?}", grid_str);
        let grid_lines: Vec<&str> = grid_str.split("\n").collect();
        // println!("grid lines: {:?}", grid_lines);
        let grid_points_str = grid_lines2points_str(&grid_lines)?;
        // println!("grid points str: {:?}", grid_points_str);
        let grid_points_f32 = grid_points_str2points_f32(&grid_points_str)?;
        // println!("grid points f32: {:?}", grid_points_f32);
        let grid: Vec<na::Vector3<f32>> = grid_points_f32to_grid(&grid_points_f32)?;
        println!("grid: {:?}", grid);
        Ok(grid)
    }

    fn add_zeros_to_edges(input: &Vec<na::Vector3<f32>>, count: i32) -> Vec<na::Vector3<f32>> {
        let mut input_zeroed: Vec<na::Vector3<f32>> = Vec::with_capacity((count * 4) as usize + input.len());
        let step = 2. / count as f32;
        let mut coord = 0.;
        for _i in 0..count {
            coord += step;
            input_zeroed.push(na::Vector3::new(-1. + coord, 0., -1.));
            input_zeroed.push(na::Vector3::new(1., 0., -1. + coord));
            input_zeroed.push(na::Vector3::new(1. - coord, 0., 1.));
            input_zeroed.push(na::Vector3::new(-1., 0., 1. - coord));
        }
        for elem in input {
            input_zeroed.push(*elem);
        }
        input_zeroed
    }

    // Makes isomorphic size*size 2d grid on [-1;1] through input points (poles)
    fn make_grid(size: usize, poles: &Vec<na::Vector3<f32>>, griding_algo: GridingAlgo) -> Vec<Vec<f32>> {
        let step: f32 = 2. / size as f32;
        let griding_function = Grid::match_griding_function(griding_algo);
        let mut cur_point: na::Vector3<f32> = na::Vector3::new(-1. - step, 0., -1. - step);
        let mut grid: Vec<Vec<f32>> = vec![vec![0.; size]; size];

        for row in &mut grid {
            cur_point.z += step;
            for elem in row {
                cur_point.x += step;
                *elem = griding_function(&cur_point, poles);
                if cur_point.x < -0.5 + 0.0001 && cur_point.x > -0.5 - 0.0001 && cur_point.z < -0.5 + 0.0001 && cur_point.z > -0.5 - 0.0001 {
                    println!("Elem: {}, p: ({}, {})", *elem, cur_point.x, cur_point.z);
                }
                if cur_point.x < -1. + 0.0001 && cur_point.x > -1. - 0.0001 && cur_point.z < -1. + 0.0001 && cur_point.z > -1. - 0.0001 {
                    println!("Elem: {}, p: ({}, {})", *elem, cur_point.x, cur_point.z);
                }
            }
            cur_point.x = -1. - step;
        }
        grid
    }

    fn match_griding_function(griding_algo: GridingAlgo) -> fn(&na::Vector3<f32>, &Vec<na::Vector3<f32>>) -> f32 {
        match griding_algo {
            GridingAlgo::Kriging => Grid::kriging_calculate_point,
            GridingAlgo::RadialBasisFunction => Grid::rbf_calculate_point,
        }
    }

    fn rbf_calculate_point(cur_point: &na::Vector3<f32>, poles: &Vec<na::Vector3<f32>>) -> f32 {
        let mut rev_distances: Vec<f32> = Vec::with_capacity(poles.len());



        for pole in poles {
            let dist = max(length_on_xz(cur_point, pole), f32::EPSILON * 100.);
            rev_distances.push(1. / dist);
        }

        let shaping_factor = 5.6 / (25. * poles.len() as f32);
        // let rbf_func = |dist :&f32| ((dist * dist) + shaping_factor).sqrt();
        // let rbf_func = |dist :&f32| 1. / ((dist * dist) + shaping_factor).sqrt();
        // let rbf_func = |dist :&f32| ((dist * dist) + shaping_factor).ln();
        let rbf_func = |dist :&f32| ((dist * dist) + shaping_factor).powf(3.).sqrt();

        let weights: Vec<f32> = rev_distances.iter().map(rbf_func).collect(); // TODO: add function to config
        let weights_sum: f32 = weights.iter().sum();

        let mut y_value = 0.;

        for (w, d) in weights.iter().zip(poles) {
            let weight = w / weights_sum;
            y_value += d.y * weight;
        }
        y_value
    }

    fn kriging_calculate_point(cur_point: &na::Vector3<f32>, poles: &Vec<na::Vector3<f32>>) -> f32 {
        let mut rev_distances: Vec<f32> = Vec::with_capacity(poles.len());
        let mut sum_rev_dists: f32 = 0.;
        for pole in poles {
            let rev_dist = 1. / max(length_on_xz(cur_point, pole), f32::MIN_POSITIVE * 100.);
            sum_rev_dists += rev_dist;
            rev_distances.push(rev_dist);
        }
        let weights: Vec<f32> = rev_distances.iter().map(|rev_dist| rev_dist / sum_rev_dists).collect();
        let mut y_value = 0.;
        for (w, d) in weights.iter().zip(poles) {
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

fn length_on_xz(p1: &na::Vector3<f32>, p2: &na::Vector3<f32>) -> f32 {
    ((p1.x - p2.x).powf(2.) + (p1.z - p2.z).powf(2.)).sqrt()
}

fn grid_str2file(str: CString, filename: &str) -> Result<String, Error> {
    str.into_string().map_err(
        |_| Error::UnableConvertFileToString { name: filename.into() }
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
            coord.trim().parse::<f32>().map_err(|e| Error::ComponentIsNotF32 { name: coord.to_string(), message: e.to_string() })
        }).collect::<Result<Vec<f32>, Error>>()
    }).collect::<Result<Vec<Vec<f32>>, Error>>()
}

fn grid_points_f32to_grid(points: &Vec<Vec<f32>>) -> Result<Vec<na::Vector3<f32>>, Error> {
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
        Ok(na::Vector3::new(x, y, z))
    }).collect::<Result<Vec<na::Vector3<f32>>, Error>>()
}
