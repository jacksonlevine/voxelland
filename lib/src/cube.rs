use crate::vec::{self};

use num_enum::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq)]
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
pub struct Cube {}
impl Cube {
    pub fn get_neighbors() -> &'static [vec::IVec3] {
        static NEIGHBORS: [vec::IVec3; 6] = [
            vec::IVec3 { x: -1, y: 0, z: 0 },
            vec::IVec3 { x: 1, y: 0, z: 0 },
            vec::IVec3 { x: 0, y: -1, z: 0 },
            vec::IVec3 { x: 0, y: 1, z: 0 },
            vec::IVec3 { x: 0, y: 0, z: -1 },
            vec::IVec3 { x: 0, y: 0, z: 1 },
        ];
        return NEIGHBORS.as_slice();
    }
    pub fn get_side(side: CubeSide) -> &'static [u8] {
        #[rustfmt::skip]
        static SIDES: [[u8; 24]; 6] = [
            [
                0, 0, 1, 12,
                0, 0, 0, 12,
                0, 1, 0, 12,
                0, 1, 0, 12,
                0, 1, 1, 12,
                0, 0, 1, 12,
            ],
            [
                1, 0, 0, 13,
                1, 0, 1, 13,
                1, 1, 1, 13,
                1, 1, 1, 13,
                1, 1, 0, 13,
                1, 0, 0, 13,
            ],
            [
                0, 0, 1, 7,
                1, 0, 1, 7,
                1, 0, 0, 7,
                1, 0, 0, 7,
                0, 0, 0, 7,
                0, 0, 1, 7,
            ],
            [
                0, 1, 0, 15,
                1, 1, 0, 15,
                1, 1, 1, 15,
                1, 1, 1, 15,
                0, 1, 1, 15,
                0, 1, 0, 15,
            ],
            [
                0, 0, 0, 10,
                1, 0, 0, 10,
                1, 1, 0, 10,
                1, 1, 0, 10,
                0, 1, 0, 10,
                0, 0, 0, 10,
            ],
            [
                1, 0, 1, 14,
                0, 0, 1, 14,
                0, 1, 1, 14,
                0, 1, 1, 14,
                1, 1, 1, 14,
                1, 0, 1, 14,
            ],
        ];

        return SIDES[side as usize].as_slice();
    }
    pub fn get_amb_occul_spots(side: CubeSide, corner: u8) -> &'static [vec::IVec3; 3]{
        #[rustfmt::skip]
        static SPOTS: [[[vec::IVec3; 3]; 6]; 6] = [
    // left
    [
        [
            vec::IVec3 { x: -1, y: -1, z: 1 },
            vec::IVec3 { x: -1, y: -1, z: 0 },
            vec::IVec3 { x: -1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: -1, y: -1, z: 0 },
            vec::IVec3 { x: -1, y: -1, z: -1 },
            vec::IVec3 { x: -1, y: 0, z: -1 }
        ],
        [
            vec::IVec3 { x: -1, y: 0, z: -1 },
            vec::IVec3 { x: -1, y: 1, z: -1 },
            vec::IVec3 { x: -1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: -1, y: 0, z: -1 },
            vec::IVec3 { x: -1, y: 1, z: -1 },
            vec::IVec3 { x: -1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: -1, y: 1, z: 0 },
            vec::IVec3 { x: -1, y: 1, z: 1 },
            vec::IVec3 { x: -1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: -1, y: -1, z: 1 },
            vec::IVec3 { x: -1, y: -1, z: 0 },
            vec::IVec3 { x: -1, y: 0, z: 1 }
        ],
    ],
    // right
    [
        [
            vec::IVec3 { x: 1, y: -1, z: 0 },
            vec::IVec3 { x: 1, y: -1, z: -1 },
            vec::IVec3 { x: 1, y: 0, z: -1 }
        ],
        [
            vec::IVec3 { x: 1, y: -1, z: 0 },
            vec::IVec3 { x: 1, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: 1, y: 0, z: 1 },
            vec::IVec3 { x: 1, y: 1, z: 1 },
            vec::IVec3 { x: 1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: 1, y: 0, z: 1 },
            vec::IVec3 { x: 1, y: 1, z: 1 },
            vec::IVec3 { x: 1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: 1, y: 1, z: 0 },
            vec::IVec3 { x: 1, y: 1, z: -1 },
            vec::IVec3 { x: 1, y: 0, z: -1 }
        ],
        [
            vec::IVec3 { x: 1, y: -1, z: 0 },
            vec::IVec3 { x: 1, y: -1, z: -1 },
            vec::IVec3 { x: 1, y: 0, z: -1 }
        ],
    ],
    // bottom
    [
        [
            vec::IVec3 { x: -1, y: -1, z: 0 },
            vec::IVec3 { x: -1, y: -1, z: 1 },
            vec::IVec3 { x: 0, y: -1, z: 1 }
        ],
        [
            vec::IVec3 { x: 0, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: -1, z: 0 }
        ],
        [
            vec::IVec3 { x: 1, y: -1, z: 0 },
            vec::IVec3 { x: 1, y: -1, z: -1 },
            vec::IVec3 { x: 0, y: -1, z: -1 }
        ],
        [
            vec::IVec3 { x: 1, y: -1, z: 0 },
            vec::IVec3 { x: 1, y: -1, z: -1 },
            vec::IVec3 { x: 0, y: -1, z: -1 }
        ],
        [
            vec::IVec3 { x: 0, y: -1, z: -1 },
            vec::IVec3 { x: -1, y: -1, z: -1 },
            vec::IVec3 { x: -1, y: -1, z: 0 }
        ],
        [
            vec::IVec3 { x: -1, y: -1, z: 0 },
            vec::IVec3 { x: -1, y: -1, z: 1 },
            vec::IVec3 { x: 0, y: -1, z: 1 }
        ],
    ],
    // top
    [
        [
            vec::IVec3 { x: -1, y: 1, z: 0 },
            vec::IVec3 { x: -1, y: 1, z: -1 },
            vec::IVec3 { x: 0, y: 1, z: -1 }
        ],
        [
            vec::IVec3 { x: 0, y: 1, z: -1 },
            vec::IVec3 { x: 1, y: 1, z: -1 },
            vec::IVec3 { x: 1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: 1, y: 1, z: 0 },
            vec::IVec3 { x: 1, y: 1, z: 1 },
            vec::IVec3 { x: 0, y: 1, z: 1 }
        ],
        [
            vec::IVec3 { x: 1, y: 1, z: 0 },
            vec::IVec3 { x: 1, y: 1, z: 1 },
            vec::IVec3 { x: 0, y: 1, z: 1 }
        ],
        [
            vec::IVec3 { x: 0, y: 1, z: 1 },
            vec::IVec3 { x: -1, y: 1, z: 1 },
            vec::IVec3 { x: -1, y: 1, z: 0 }
        ],
        [
            vec::IVec3 { x: -1, y: 1, z: 0 },
            vec::IVec3 { x: -1, y: 1, z: -1 },
            vec::IVec3 { x: 0, y: 1, z: -1 }
        ],
    ],
    // back
    [
        [
            vec::IVec3 { x: -1, y: 0, z: -1 },
            vec::IVec3 { x: -1, y: -1, z: -1 },
            vec::IVec3 { x: 0, y: -1, z: -1 }
        ],
        [
            vec::IVec3 { x: 0, y: -1, z: -1 },
            vec::IVec3 { x: 1, y: -1, z: -1 },
            vec::IVec3 { x: 1, y: 0, z: -1 }
        ],
        [
            vec::IVec3 { x: 1, y: 0, z: -1 },
            vec::IVec3 { x: 1, y: 1, z: -1 },
            vec::IVec3 { x: 0, y: 1, z: -1 }
        ],
        [
            vec::IVec3 { x: 1, y: 0, z: -1 },
            vec::IVec3 { x: 1, y: 1, z: -1 },
            vec::IVec3 { x: 0, y: 1, z: -1 }
        ],
        [
            vec::IVec3 { x: 0, y: 1, z: -1 },
            vec::IVec3 { x: -1, y: 1, z: -1 },
            vec::IVec3 { x: -1, y: 0, z: -1 }
        ],
        [
            vec::IVec3 { x: -1, y: 0, z: -1 },
            vec::IVec3 { x: -1, y: -1, z: -1 },
            vec::IVec3 { x: 0, y: -1, z: -1 }
        ],
    ],
    // front
    [
        [
            vec::IVec3 { x: 0, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: 0, y: -1, z: 1 },
            vec::IVec3 { x: -1, y: -1, z: 1 },
            vec::IVec3 { x: -1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: -1, y: 0, z: 1 },
            vec::IVec3 { x: -1, y: 1, z: 1 },
            vec::IVec3 { x: 0, y: 1, z: 1 }
        ],
        [
            vec::IVec3 { x: -1, y: 0, z: 1 },
            vec::IVec3 { x: -1, y: 1, z: 1 },
            vec::IVec3 { x: 0, y: 1, z: 1 }
        ],
        [
            vec::IVec3 { x: 0, y: 1, z: 1 },
            vec::IVec3 { x: 1, y: 1, z: 1 },
            vec::IVec3 { x: 1, y: 0, z: 1 }
        ],
        [
            vec::IVec3 { x: 0, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: -1, z: 1 },
            vec::IVec3 { x: 1, y: 0, z: 1 }
        ],
    ],
];
        return &SPOTS[side as usize][corner as usize];
    }
}
