











pub struct SpecialBlocks {

}

impl SpecialBlocks {

    pub fn contains(int: u32) -> bool {
        static SPECIAL_BLOCKS: [u32; 1] = [
            19
        ];

        return SPECIAL_BLOCKS.contains(&int);
    }

    

}