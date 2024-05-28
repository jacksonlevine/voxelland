use crate::cube::CubeSide;

pub struct Blocks {}

static TEXS: [[(u8, u8); 3]; 19] = [
            //sides   //bot   //top
            [(0, 0), (0, 0), (0, 0)],  // 0
            [(1, 0), (1, 0), (1, 0)],  // 1 sand
            [(2, 0), (2, 0), (2, 0)],  // 2 water
            [(3, 0), (4, 0), (3, 1)],  // 3 grass
            [(4, 0), (4, 0), (4, 0)],  // 4 dirt
            [(5, 0), (5, 0), (5, 0)],  // 5 cobble
            [(6, 0), (6, 1), (6, 1)],  // 6 log
            [(7, 0), (7, 0), (7, 0)],  // 7 leaves
            [(8, 0), (8, 0), (8, 0)],    // 08 glass
            [(9, 0), (9, 0), (9, 0)],    // 09 smooth stone
            [(10, 0), (10, 0), (10, 0)], // 10 planks wood
            [(7, 1), (7, 1), (7, 1)], // 11 bush leaves
            [(4, 2), (4, 2), (4, 2)], // 12 petrified wood
            [(6, 2), (6, 2), (6, 2)], // 13 red stone
            [(7, 2), (7, 2), (7, 2)], // 14 salted earth
            [(8, 2), (8, 2), (8, 2)], // 15 bedrock
            [(0, 3), (0, 3), (0, 3)], // 16 red crystal unattainable
            [(0, 4), (0, 4), (0, 4)], // 17 red crystal
            [(12, 1), (12, 1), (12, 1)], // 18 light
        ];

static BREAKTIMES: [f32; 18] = [
    0.1,
    0.5,
    0.7,
    0.7,
    0.5,
    1.0,
    0.7,
    0.2,
    0.7,
    1.5,
    0.7,
    0.8,
    1.1,
    1.5,
    0.7,
    9999999.0,
    1.2,
    0.5
];

impl Blocks {
    pub fn get_break_time(id: u32) -> f32 {
        return BREAKTIMES[id as usize];
    }
    pub fn get_texs_length() -> usize {
        return TEXS.len();
    }
    pub fn get_tex_coords(id: u32, side: CubeSide) -> &'static (u8, u8) {
        static SIDES: [usize; 6] = [0, 0, 1, 2, 0, 0];
        
        return &TEXS[id as usize][SIDES[side as usize]];
    }
    pub fn is_transparent(id: u32) -> bool {
        static TRANSPARENTS: [u32; 2] = [
            2, 8,
        ];
        return TRANSPARENTS.contains(&id);
    }
    pub fn is_semi_transparent(id: u32) -> bool {
        static SEMI_TRANSPARENTS: [u32; 2] = [
            7, 11,
        ];
        return SEMI_TRANSPARENTS.contains(&id);
    }
    pub fn is_light(id: u32) -> bool {
        static LIGHTS: [u32; 1] = [
            18
        ];
        return LIGHTS.contains(&id);
    }
    pub fn get_walk_series(id: u32) -> &'static str {
        match id {
            3 => {
                "grassstepseries"
            }
            _ => {
                "stonestepseries"
            }
        }
    }
}
