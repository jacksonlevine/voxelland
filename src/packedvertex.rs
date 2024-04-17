


pub struct PackedVertex {

}


impl PackedVertex {
    pub fn pack(x: u8, y: u8, z: u8, corner: u8, al: u8, bl: u8, u: u8, v: u8, ) -> (u32, u8) {
        let shifted_x = (x as u32) << 28;
        let shifted_y = (y as u32) << 24;
        let shifted_cropped_z = (((z as u32) & 0b0000_0000_0000_0000_0000_0000_0000_1111) << 16);
        let shifted_corner = (corner as u32) << 12;
        let shifted_amb = (al as u32) << 8;
        let shifted_block = (bl as u32) << 4;


        let mut sum32_bits: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0000;
        sum32_bits |= shifted_x;
        sum32_bits |= shifted_y;
        sum32_bits |= shifted_cropped_z;
        sum32_bits |= shifted_corner;
        sum32_bits |= shifted_amb;
        sum32_bits |= shifted_block;

        let shifted_u = (u as u8) << 4;
        let shifted_v = (v as u8) << 0;

        let mut sum8_bits: u8 = 0b0000_0000;
        sum8_bits |= shifted_u;
        sum8_bits |= shifted_v;

        (sum32_bits, sum8_bits)
    }
}