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

        // let view_rotation: na::Matrix4<f32> = na::Isometry3::rotation(na::Vector3::y() * 0.).to_homogeneous();
        // let view_translation: na::Matrix4<f32> = na::Isometry3::translation(0., 0., -3.).to_homogeneous();

        let eye = na::Point3::new(0., 0., 2.);
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

    pub fn get_transform(&self) -> na::Matrix4<f32> {
        self.projection * self.view * self.model
    }

    pub fn recalc_projection(&mut self, w: i32, h: i32) {
        let aspect: f32 = (w) as f32 / (h) as f32;
        println!("aspect: {}", aspect);
        self.projection = na::Perspective3::new(aspect, 3.14 / 2.0, 1.0, 1_000.0)
            .to_homogeneous()
    }

    pub fn recalc_model_naviball(&mut self, naviball: na::Vector2<f32>) {
        let rot_y = na::Isometry3::rotation(na::Vector3::y() * 3.14 * naviball.x);
        let rot_x = na::Isometry3::rotation(na::Vector3::x() * 3.14 * naviball.y);
        let rot_total: na::Matrix4<f32> = (rot_x * rot_y).to_homogeneous();
        self.model = rot_total * self.model;
    }
}