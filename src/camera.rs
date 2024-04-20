use glam::{Vec3, Mat4};
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
        let fov: f32 = 90.0;
        let far = 1000.0;
        let near = 0.01;
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
            near
        }
    }
}