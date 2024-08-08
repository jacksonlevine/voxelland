use crate::game::ROWLENGTH;

pub struct Inventory {
    pub dirty: bool,
    pub inv: [(u32, u32); ROWLENGTH as usize]
}

pub struct ChestInventory {
    pub dirty: bool,
    pub inv: [(u32, u32); ROWLENGTH as usize * 4]
}