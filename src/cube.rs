
use crate::vec;
use num_enum::{FromPrimitive};

#[derive(Debug, Clone, Copy, FromPrimitive)]
#[repr(usize)]
pub enum CubeSide {
    #[num_enum(default)]
    LEFT = 0,
    RIGHT = 1,
    BOTTOM = 2,
    TOP = 3,
    BACK = 4,
    FRONT = 5,
}
pub struct Cube {
    
}
impl Cube {
    pub fn get_neighbors() -> &'static[vec::IVec3] {
        static NEIGHBORS: [vec::IVec3; 6] = [
            vec::IVec3{x:-1, y:0, z:0},
            vec::IVec3{x:1, y:0, z:0},
            vec::IVec3{x:0, y:-1, z:0},
            vec::IVec3{x:0, y:1, z:0},
            vec::IVec3{x:0, y:0, z:-1},
            vec::IVec3{x:0, y:0, z:1},
        ];
        return NEIGHBORS.as_slice();
    }
    pub fn get_side(side: CubeSide) -> &'static[u8] {
        static SIDES: [[u8; 18]; 6] = [
        [0, 0, 1
        , 0, 0, 0
        , 0, 1, 0
        , 0, 1, 0
        , 0, 1, 1
        , 0, 0, 1
        ],
            [1, 0, 0
        , 1, 0, 1
        , 1, 1, 1
        , 1, 1, 1
        , 1, 1, 0
        , 1, 0, 0
        ],
            [0, 0, 1
        , 1, 0, 1
        , 1, 0, 0
        , 1, 0, 0
        , 0, 0, 0
        , 0, 0, 1
        ],
            [0, 1, 0
        , 1, 1, 0
        , 1, 1, 1
        , 1, 1, 1
        , 0, 1, 1
        , 0, 1, 0
        ],
            [0, 0, 0
        , 1, 0, 0
        , 1, 1, 0
        , 1, 1, 0
        , 0, 1, 0
        , 0, 0, 0
        ],
            [1, 0, 1
        , 0, 0, 1
        , 0, 1, 1
        , 0, 1, 1
        , 1, 1, 1
        , 1, 0, 1
        ],
        ];

        return SIDES[side as usize].as_slice();
    }
}