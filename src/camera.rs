use na;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct MVP {
    model: na::Matrix4<f32>,
    view: na::Matrix4<f32>,
    projection: na::Matrix4<f32>,
}

impl MVP {
    pub fn new() -> MVP {

        let model: na::Matrix4<f32> = na::Isometry3::identity()
            .to_homogeneous();

        let eye = na::Point3::new(0., 0., -2.);
        let target = na::Point3::new(0., 0., 0.);
        let view: na::Matrix4<f32> = na::Isometry3::look_at_rh(&eye, &target, &na::Vector3::y())
            .to_homogeneous();

        let projection: na::Matrix4<f32> = na::Perspective3::new(9.0 / 7.0, 3.14 / 2.0, 1.0, 1_000.0)
            .to_homogeneous();

        MVP {
            model,
            view,
            projection,
        }
    }

    pub fn rotate_left(&mut self) {
        let rot: na::Matrix4<f32> = na::Isometry3::rotation(na::Vector3::y() * 3.14 / 20.).to_homogeneous();
        self.model = self.model * rot;
    }

    pub fn get_transform(&self) -> na::Matrix4<f32> {
        self.projection * self.view * self.model
    }
}