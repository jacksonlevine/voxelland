use glam::{Vec3};

use crate::vec;
use num_enum::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq)]
#[repr(usize)]
pub enum Side {
    #[num_enum(default)]
    ROOF = 0,
    FLOOR,

    LEFTTOP,
    LEFTBOTTOM,

    RIGHTTOP,
    RIGHTBOTTOM,

    FRONTTOP,
    FRONTBOTTOM,

    BACKTOP,
    BACKBOTTOM,

    BACKRIGHTTOP,
    BACKRIGHTBOTTOM,

    BACKLEFTTOP,
    BACKLEFTBOTTOM,

    FRONTRIGHTTOP,
    FRONTRIGHTBOTTOM,

    FRONTLEFTTOP,
    FRONTLEFTBOTTOM,

    INBOTTOM,
    INTOP,

    BOTTOMFRONTEDGE,
    BOTTOMLEFTEDGE,
    BOTTOMRIGHTEDGE,
    BOTTOMBACKEDGE,
}

pub struct BoundBox {
    center: Vec3,
    min_corner: Vec3,
    max_corner: Vec3,
}

pub struct CollCage {
    pub num_boxes: i32,
    pub colliding: Vec<Side>,
    pub solid: Vec<Side>,
    pub position: glam::IVec3,
    pub boxes: Vec<BoundBox>,
    pub penetrations: Vec<f32>,
    pub normals: Vec<Vec3>,
    pub positions: Vec<glam::IVec3>,
    pub solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync>,
    pub smoothed_y_offset: f32,
}

impl BoundBox {
    pub fn new(center: Vec3) -> BoundBox {
        BoundBox {
            center,
            min_corner: center + Vec3::new(0.0, 0.0, 0.0),
            max_corner: center + Vec3::new(1.0, 1.0, 1.0),
        }
    }
    pub fn set_center(&mut self, center: Vec3, xextent: f32, yextent: f32) {
        self.min_corner = center + Vec3::new(-xextent, -yextent, -xextent);
        self.max_corner = center + Vec3::new(xextent, yextent, xextent);
        self.center = center;
    }
    pub fn set_center_block(&mut self, center: Vec3) {
        self.min_corner = center + Vec3::new(0.0, 0.0, 0.0);
        self.max_corner = center + Vec3::new(1.0, 1.0, 1.0);
        self.center = center;
    }
    pub fn intersects(&self, other: &BoundBox) -> bool {
        return !(self.max_corner.x < other.min_corner.x
            || self.min_corner.x > other.max_corner.x
            || self.max_corner.y < other.min_corner.y
            || self.min_corner.y > other.max_corner.y
            || self.max_corner.z < other.min_corner.z
            || self.min_corner.z > other.max_corner.z);
    }
    pub fn get_penetration(&self, other: &BoundBox) -> f32 {
        if !self.intersects(other) {
            return 0.0;
        } else {
            let x_penetration = f32::min(
                self.max_corner.x - other.min_corner.x,
                other.max_corner.x - self.min_corner.x,
            );
            let y_penetration = f32::min(
                self.max_corner.y - other.min_corner.y,
                other.max_corner.y - self.min_corner.y,
            );
            let z_penetration = f32::min(
                self.max_corner.z - other.min_corner.z,
                other.max_corner.z - self.min_corner.z,
            );

            f32::min(f32::min(x_penetration, y_penetration), z_penetration)
        }
    }
}

impl CollCage {
    pub fn new(solid_pred: Box<dyn Fn(vec::IVec3) -> bool  + Send + Sync> ) -> CollCage {
        let num_boxes = 18;
        let colliding: Vec<Side> = Vec::new();
        let solid: Vec<Side> = Vec::new();
        let position = glam::IVec3::new(0, 0, 0);

        let penetrations: Vec<f32> = vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let normals = vec![
            Vec3::new(0.0, -1.0, 0.0), // TOp/roof
            Vec3::new(0.0, 1.0, 0.0),  // BOTTOM/floor
            Vec3::new(1.0, 0.0, 0.0),  // LEFTTOP
            Vec3::new(1.0, 0.0, 0.0),  // LEFTBOTTOM
            Vec3::new(-1.0, 0.0, 0.0), // RIGHTTOP
            Vec3::new(-1.0, 0.0, 0.0), // RIGHTBOTTOM
            Vec3::new(0.0, 0.0, -1.0), // FRONTTOP
            Vec3::new(0.0, 0.0, -1.0), // FRONTBOTTOM
            Vec3::new(0.0, 0.0, 1.0),  // BACKTOP
            Vec3::new(0.0, 0.0, 1.0),  // BACKBOTTOM
            Vec3::new(0.0, 0.0, 1.0),  //BACKRIGHTTOP,
            Vec3::new(0.0, 0.0, 1.0),  //BACKRIGHTBOTTOM,
            Vec3::new(0.0, 0.0, 1.0),  //BACKLEFTTOP,
            Vec3::new(0.0, 0.0, 1.0),  //BACKLEFTBOTTOM,
            Vec3::new(0.0, 0.0, -1.0), //FRONTRIGHTTOP,
            Vec3::new(0.0, 0.0, -1.0), //FRONTRIGHTBOTTOM,
            Vec3::new(0.0, 0.0, -1.0), //FRONTRIGHTTOP,
            Vec3::new(0.0, 0.0, -1.0), //FRONTRIGHTBOTTOM,
            Vec3::new(0.0, 1.0, 0.0),  //INBOTTOM
            Vec3::new(0.0, 1.0, 0.0),  //INTOP
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let positions = vec![
            glam::IVec3::new(0, 2, 0),   // TOp/roof
            glam::IVec3::new(0, -1, 0),  // BOTTOM/floor
            glam::IVec3::new(-1, 1, 0),  // LEFTTOP
            glam::IVec3::new(-1, 0, 0),  // LEFTBOTTOM
            glam::IVec3::new(1, 1, 0),   // RIGHTTOP
            glam::IVec3::new(1, 0, 0),   // RIGHTBOTTOM
            glam::IVec3::new(0, 1, 1),   // FRONTTOP
            glam::IVec3::new(0, 0, 1),   // FRONTBOTTOM
            glam::IVec3::new(0, 1, -1),  // BACKTOP
            glam::IVec3::new(0, 0, -1),  // BACKBOTTOM
            glam::IVec3::new(1, 1, -1),  //BACKRIGHTTOP,
            glam::IVec3::new(1, 0, -1),  //BACKRIGHTBOTTOM,
            glam::IVec3::new(-1, 1, -1), //BACKLEFTTOP,
            glam::IVec3::new(-1, 0, -1), //BACKLEFTBOTTOM,
            glam::IVec3::new(1, 1, 1),   //FRONTRIGHTTOP,
            glam::IVec3::new(1, 0, 1),   //FRONTRIGHTBOTTOM,
            glam::IVec3::new(-1, 1, 1),  //FRONTLEFTTOP,
            glam::IVec3::new(-1, 0, 1),  //FRONTLEFTBOTTOM
            glam::IVec3::new(0, 0, 0),   //inbottom
            glam::IVec3::new(0, 1, 0),   //intop
            glam::IVec3::new(0, -1, 1),
            glam::IVec3::new(-1, -1, 0),
            glam::IVec3::new(1, -1, 0),
            glam::IVec3::new(0, -1, -1),
        ];
        let boxes: Vec<BoundBox> = vec![
            BoundBox::new(positions[0].as_vec3()),
            BoundBox::new(positions[1].as_vec3()),
            BoundBox::new(positions[2].as_vec3()),
            BoundBox::new(positions[3].as_vec3()),
            BoundBox::new(positions[4].as_vec3()),
            BoundBox::new(positions[5].as_vec3()),
            BoundBox::new(positions[6].as_vec3()),
            BoundBox::new(positions[7].as_vec3()),
            BoundBox::new(positions[8].as_vec3()),
            BoundBox::new(positions[9].as_vec3()),
            BoundBox::new(positions[10].as_vec3()),
            BoundBox::new(positions[11].as_vec3()),
            BoundBox::new(positions[12].as_vec3()),
            BoundBox::new(positions[13].as_vec3()),
            BoundBox::new(positions[14].as_vec3()),
            BoundBox::new(positions[15].as_vec3()),
            BoundBox::new(positions[16].as_vec3()),
            BoundBox::new(positions[17].as_vec3()),
            BoundBox::new(positions[18].as_vec3()),
            BoundBox::new(positions[19].as_vec3()),
            BoundBox::new(positions[20].as_vec3()),
            BoundBox::new(positions[21].as_vec3()),
            BoundBox::new(positions[22].as_vec3()),
            BoundBox::new(positions[23].as_vec3()),
        ];

        CollCage {
            num_boxes,
            colliding,
            solid,
            position,
            boxes,
            penetrations,
            normals,
            positions,
            solid_pred,
            smoothed_y_offset: 0.0
        }
    }

    pub fn get_smoothed_floor_y(&mut self, actualpos: Vec3) -> f32 {
        // Get the integer coordinates of the current square
        let square_x = actualpos.x.floor() as i32;
        let square_z = actualpos.z.floor() as i32;
    
        // Initialize variables to store heights of neighboring blocks
        let mut neg_x_height = 0.0;
        let mut pos_x_height = 0.0;
        let mut neg_z_height = 0.0;
        let mut pos_z_height = 0.0;
    
        // Check if neighboring blocks are present and set their heights accordingly
        if self.solid.contains(&Side::LEFTBOTTOM) { neg_x_height = 1.0; }
        if self.solid.contains(&Side::RIGHTBOTTOM) { pos_x_height = 1.0; }
        if self.solid.contains(&Side::BACKBOTTOM) { neg_z_height = 1.0; }
        if self.solid.contains(&Side::FRONTBOTTOM) { pos_z_height = 1.0; }
    
        // Get the fractional position within the square
        let frac_x = actualpos.x - square_x as f32;
        let frac_z = actualpos.z - square_z as f32;
    
        // Interpolate between neighboring block heights based on fractional position
        let lerped_height_x = neg_x_height * (1.0 - frac_x) + pos_x_height * frac_x;
        let lerped_height_z = neg_z_height * (1.0 - frac_z) + pos_z_height * frac_z;
    
        // Interpolate between lerped heights based on the dominant axis of movement
        let dominant_axis = if frac_x.abs() > frac_z.abs() { lerped_height_x } else { lerped_height_z };
    
        // Return the smoothed Y position by adding an offset to the dominant axis height
        return actualpos.y.floor() + dominant_axis;
    }
    pub fn update_readings(&mut self, pos: Vec3) {
        self.update_position(pos);
        self.update_solidity();
    }
    pub fn update_colliding(&mut self, user: &BoundBox) {
        //Reset
        self.colliding.clear();
        for i in 0..self.num_boxes {
            self.penetrations[i as usize] = 0.0;
        }

        //Reassess
        for side in &self.solid {
            if user.intersects(&self.boxes[*side as usize]) {
                if !self.colliding.contains(&side) {
                    self.colliding.push(*side);
                }
            }
            self.penetrations[*side as usize] = user.get_penetration(&self.boxes[*side as usize]);
        }
    }
    pub fn update_solidity(&mut self) {
        self.solid.clear();
        for i in 0..self.num_boxes {
            let spot = self.boxes[i as usize].center;
            let tup = vec::IVec3 {
                x: spot.x as i32,
                y: spot.y as i32,
                z: spot.z as i32,
            };
            let side = Side::from_primitive(i as usize);
            if (self.solid_pred)(tup) {
                if !self.solid.contains(&side) {
                    self.solid.push(side);
                }
            } else {
                self.solid.retain(|&x| x != side);
            }
        }
    }
    pub fn update_position(&mut self, pos: Vec3) {
        self.position = glam::IVec3::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        for i in 0..self.num_boxes {
            let new_center = self.positions[i as usize] + self.position;
            self.boxes[i as usize].set_center_block(Vec3::new(
                new_center.x as f32,
                new_center.y as f32,
                new_center.z as f32,
            ));
        }
    }
}
