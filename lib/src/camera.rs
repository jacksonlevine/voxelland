use crate::game::{ControlsState, MOVING, SPRINTING};
use glam::{Mat4, Vec3};

#[derive(Clone, Default)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,

    pub direction: Vec3,
    pub position: Vec3,
    pub right: Vec3,
    pub up: Vec3,

    pub model: Mat4,
    pub projection: Mat4,
    pub view: Mat4,
    pub mvp: Mat4,

    pub velocity: Vec3,

    pub far: f32,
    pub near: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let position = Vec3::new(0.0, 100.0, 0.0);
        let right = Vec3::new(0.0, 1.0, 0.0).cross(direction).normalize();
        let fov: f32 = 80.0;
        let far = 560.0;
        let near = 0.001;
        let up = direction.cross(right);

        let model = Mat4::IDENTITY;
        let projection = Mat4::perspective_rh_gl(fov.to_radians(), 1280.0 / 720.0, near, far);
        let view = Mat4::look_at_rh(position, position + direction, up);
        Camera {
            yaw: 0.0,
            pitch: 0.0,
            fov,
            direction,
            position: Vec3::new(0.0, 100.0, 0.0),
            right,
            up: direction.cross(right),
            model,
            projection,
            view,
            mvp: projection * model * view,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            far,
            near,
        }
    }
    pub fn update_fov(&mut self, value: f32) {
        self.fov = value.clamp(50.0, 160.0);
        self.projection =
            Mat4::perspective_rh_gl(self.fov.to_radians(), 1280.0 / 720.0, self.near, self.far);
        self.recalculate();
    }
    pub fn recalculate(&mut self) {
        self.right = Vec3::new(0.0, 1.0, 0.0).cross(self.direction).normalize();
        self.up = self.direction.cross(self.right);
        self.view = Mat4::look_at_rh(self.position, self.position + self.direction, self.up);
        self.mvp = self.projection * self.view * self.model;
    }
    pub fn respond_to_controls(
        &mut self,
        cs: &ControlsState,
        delta: &f32,
        speed_mult: f32,
    ) -> Vec3 {

        let mut xz_speed_mult = 2.2;
        unsafe {
            if SPRINTING  {
                xz_speed_mult = 2.74;
            }
        }
        
        let mut moving = false;

        if cs.forward {
            moving = true;
            self.velocity += (self.direction * Vec3::new(1.0, 0.0, 1.0)).normalize() * xz_speed_mult * *delta * speed_mult;
        }
        if cs.left {
            moving = true;
            self.velocity += (self.right * Vec3::new(xz_speed_mult, 0.0, xz_speed_mult)) * *delta * speed_mult;
        }
        if cs.back {
            moving = true;
            self.velocity += (self.direction * Vec3::new(1.0, 0.0, 1.0)).normalize() * xz_speed_mult * -*delta * speed_mult;
        }
        if cs.right {
            moving = true;
            self.velocity += (self.right * Vec3::new(xz_speed_mult, 0.0, xz_speed_mult)) * -*delta * speed_mult;
        }
        unsafe {
            MOVING = moving;
        }
        self.recalculate();

        //let closeness_to_stopped = (0.7 - Vec3::new(self.velocity.x, 0.0, self.velocity.z).length()).max(0.0);

        let slipperiness: f32 = 0.3;

        self.velocity.x *= slipperiness.powf(*delta * speed_mult);
        self.velocity.z *= slipperiness.powf(*delta * speed_mult);

        if self.velocity.length() > 0.0 {
            let amt_to_subtract = self.velocity * *delta * speed_mult;

            self.velocity -= amt_to_subtract;

            return self.position + amt_to_subtract;
        } else {
            return self.position;
        }

        #[cfg(feature = "show_cam_pos")]
        info!(
            "Cam pos: {}, {}, {}",
            self.position.x, self.position.y, self.position.z
        );
    }
}
