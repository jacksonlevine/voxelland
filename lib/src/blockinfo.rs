use crate::{chunk::LightColor, cube::CubeSide};

pub const BLOCK_DIRECTION_BITS: u32 = 0b0000_0000_0000_0011_0000_0000_0000_0000;
pub struct Blocks {}


static BREAKTIMES: [f32; 47] = [
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
    0.5,
    1.0,
    1.0,
    0.6,
    1.5,
    1.0,
    0.2,

    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,

    1.0, 
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,

    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    0.5,
    0.5
];

static TEXS: [[(u8, u8); 3]; 47] = [
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

            [(12, 0), (12, 0), (12, 0)], // 19 door
            [(0, 1), (0, 1), (0, 1)], // 20 ladder
            [(15, 0), (15, 0), (15, 0)], // 21 wooden trunk
            [(13, 1), (14, 1), (14, 1)], // 22 bamboo
            [(1, 3), (1, 3), (1, 3)], // 23 tallgrass

            [(10, 2), (10, 2), (10, 2)], // 24 blue light
            [(11, 2), (11,2), (11, 2)], // 25 purple light
            [(12, 2), (12, 2), (12, 2)], // 26 yellow light

            [(13, 2), (13, 2), (13, 2)], // 27 red light
            [(10, 3), (10, 3), (10, 3)], // 28 green light
            [(11, 3), (11, 3), (11, 3)], // 29 orange light
            [(12, 3), (12, 3), (12, 3)], // 30 teal light
            [(1,5), (1,5), (1,5)], // 31 crafttable

            [(3, 3), (3, 3), (3, 3)], // 32 apple
            [(2, 3),(2, 3),(2, 3)], // 33 bamboo chute
            [(7,4),(7,4),(7,4)], // 34 dead leaves


            [(2,4),(2,4),(2,4)], // 35 metal rock
            [(2,5),(2,5),(2,5)], // 36 crude blade

            [(3,5),(3,5),(3,5)], // 37 crude pick
            [(4,5),(4,5),(4,5)], // 38 crude mattock
            [(5,5),(5,5),(5,5)], // 39 crude axe


            [(10,4),(10,4),(10,4)], // 40 jumper blue
            [(11,4),(11,4),(11,4)], // 41 jumper yellow
            [(10,5),(10,5),(10,5)], // 42 trampoline block

            [(0,8),(2,8),(2,8)], // 43 rubber tree wood
            [(1,8),(1,8),(1,8)], // 44 rubber tree leaves
            [(10,6),(10,6),(10,6)], // 45 conveyor/highway
            [(11,5),(11,5),(11,5)], // 46 auto trampoline block
        ];


impl Blocks {
    pub fn get_name(id: u32) -> &'static str {
        match id {
            0 => {"Air"}
            1 => {"Sand"}
            2 => {"Water"}
            3 => {"Grass"}
            4 => {"Dirt"}
            5 => {"Cobblestone"}
            6 => {"Wood"}
            7 => {"Leaves"}
            8 => {"Glass"}
            9 => {"Stone"}
            10 => {"Wood Planks"}
            11 => {"Bush Leaves"}
            12 => {"Petrified Wood"}
            13 => {"Red Stone"}
            14 => {"Salted Earth"}
            15 => {"Bedrock"}
            16 => {"Red Crystal Unattainable"}
            17 => {"Red Crystal"}
            18 => {"Light"}
            19 => {"Door"}
            20 => {"Ladder"}
            21 => {"Wooden Trunk"}
            22 => {"Bamboo"}
            23 => {"Tall Grass"}
            24 => {"Blue Light"}
            25 => {"Purple Light"}
            26 => {"Yellow Light"}
            27 => {"Red Light"}
            28 => {"Green Light"}
            29 => {"Orange Light"}
            30 => {"Teal Light"}
            31 => {"Crafting Bench"}

            32 => {"Apple"}
            33 => {"Bamboo Piece"}
            34 => {"Dead Leaf Mulch"}
            35 => {"Metal Rock"}
            36 => {"Crude Blade"}

            37 => {"Crude Pick"}
            38 => {"Crude Mattock"}
            39 => {"Crude Axe"}

            40 => {"Jump Switcher Block"}
            41 => {"Jump Switcher Block"}
            42 => {"Trampoline Block"}

            43 => {"Rubber Tree Wood"}
            44 => {"Rubber Tree Leaves"}
            45 => {"Conveyor"}
            46 => {"Auto Trampoline"}
            _ => {
                "Unknown Item"
            }
        }
    }
    pub fn get_light_color(id: u32) -> LightColor {
        static white: LightColor = LightColor{x: 15, y: 15, z:15};
        static blue: LightColor = LightColor{x: 0, y:0, z:15};
        static purple: LightColor = LightColor{x: 7, y:0, z:10};
        static yellow: LightColor = LightColor{x: 15, y:15, z:0};

        static red: LightColor = LightColor{x: 15, y:0, z:0};
        static green: LightColor = LightColor{x: 0, y:15, z:0};
        static orange: LightColor = LightColor{x: 15, y:7, z:0};
        static teal: LightColor = LightColor{x: 2, y:15, z:12};

        match id {
            18 => {
                white
            }
            24 => {
                blue
            }
            25 => {
                purple
            }
            26 => {
                yellow
            }

            27 => {
                red
            }
            28 => {
                green
            }
            29 => {
                orange
            }
            30 => {
                teal
            }
            _ => {
                white
            }
        }
    }
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
    pub fn is_climbable(id: u32) -> bool {
        static CLIMBABLES: [u32; 2] = [
            20, 22
        ];
        return CLIMBABLES.contains(&id);
    }
    pub fn is_semi_transparent(id: u32) -> bool {
        static SEMI_TRANSPARENTS: [u32; 9] = [
            7, 11, 19, 20, 21, 22, 23, 31, 44
        ];
        return SEMI_TRANSPARENTS.contains(&id);
    }
    pub fn is_non_placeable(id: u32) -> bool {
        static NP: [u32; 7] = [
            32, 33, 17, 36, 37, 38, 39
        ];
        return NP.contains(&id);
    }
    pub fn is_light(id: u32) -> bool {
        static LIGHTS: [u32; 8] = [
            18, 24, 25, 26, 27, 28, 29, 30
        ];
        return LIGHTS.contains(&id);
    }
    pub fn is_food(id: u32) -> bool {
        static FOOD: [u32; 2] = [
            32, 33
        ];
        return FOOD.contains(&id);
    }

    pub fn block_id_bits() -> u32 {
        0b0000_0000_0000_0000_1111_1111_1111_1111
    }

    pub fn get_direction_bits(input: u32) -> u32 {
        return (input & BLOCK_DIRECTION_BITS) >> 16;
    }

    pub fn set_direction_bits(input: &mut u32, direction: u32) {
        let bits = direction << 16;
        *input |= bits;
    }



    pub fn block_flag_bits() -> u32 {
        0b1111_1111_1111_1111_0000_0000_0000_0000
    }
    pub fn get_food_stats(id: u32) -> (i32, i32) {
        match id {
            _ => {
                (6, 50)
            }
        }
    }
    pub fn get_walk_series(id: u32) -> &'static str {
        match id {
            3 => {
                "grassstepseries"
            }
            34 => {
                "mulchstepseries"
            }
            7 => {
                "plantplaceseries"
            }
            11 => {
                "plantplaceseries"
            }
            1 => {
                "sandstepseries"
            }
            6 => {
                "woodstepseries"
            }
            4 => {
                "dirtstepseries"
            }
            10 => {
                "woodstepseries"
            }
            22 => {
                "grassstepseries"
            }
            2 => {
                "waterstepseries"
            }
            _ => {
                "stonestepseries"
            }
        }
    }
    pub fn get_place_series(id: u32) -> &'static str {
        match id {
            3 => {
                "grassstepseries"
            }
            34 => {
                "mulchstepseries"
            }
            7 => {
                "plantplaceseries"
            }
            8 => {
                "glassplaceseries"
            }
            22 => {
                "plantplaceseries"
            }
            18 => {
                "glassplaceseries"
            }
            19 => {
                "doorseries"
            }
            11 => {
                "plantplaceseries"
            }
            _ => {
                "stoneplaceseries"
            }
        }
    }
}
