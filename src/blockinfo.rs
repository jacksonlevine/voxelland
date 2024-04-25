use crate::cube::CubeSide;

pub struct Blocks {}

impl Blocks {
    pub fn get_tex_coords(id: u32, side: CubeSide) -> &'static (u8, u8) {
        static SIDES: [usize; 6] = [0, 0, 1, 2, 0, 0];
        static TEXS: [[(u8, u8); 3]; 11] = [
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
        ];
        return &TEXS[id as usize][SIDES[side as usize]];
    }
}
