use crate::cube::CubeSide;

pub struct Blocks {}

static TEXS: [[(u8, u8); 3]; 12] = [
            //sides   //bot   //top
            [(0, 0), (0, 0), (0, 0)],
            [(1, 0), (1, 0), (1, 0)],
            [(2, 0), (2, 0), (2, 0)],
            [(3, 0), (4, 0), (3, 1)],
            [(4, 0), (4, 0), (4, 0)],
            [(5, 0), (5, 0), (5, 0)],
            [(6, 0), (6, 1), (6, 1)],
            [(7, 0), (7, 0), (7, 0)],
            [(8, 0), (8, 0), (8, 0)],
            [(9, 0), (9, 0), (9, 0)],
            [(10, 0), (10, 0), (10, 0)],
            [(7, 1), (7, 1), (7, 1)], // 11 bush leaves
        ];

impl Blocks {
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
        static SEMI_TRANSPARENTS: [u32; 7] = [
            7, 11, 14, 19, 20, 21, 22
        ];
        return SEMI_TRANSPARENTS.contains(&id);
    }
}
