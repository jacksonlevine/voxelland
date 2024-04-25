use std::hash::Hash;
use std::ops::{Add, Sub};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl Add for IVec2 {
    type Output = IVec2;

    fn add(self, rhs: Self) -> Self::Output {
        IVec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for IVec2 {
    type Output = IVec2;

    fn sub(self, rhs: Self) -> Self::Output {
        IVec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct IVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Add for IVec3 {
    type Output = IVec3;

    fn add(self, rhs: Self) -> Self::Output {
        IVec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for IVec3 {
    type Output = IVec3;

    fn sub(self, rhs: Self) -> Self::Output {
        IVec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
