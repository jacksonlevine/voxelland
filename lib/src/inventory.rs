use crate::game::ROWLENGTH;

#[derive(Clone, PartialEq)]
pub struct Inventory {
    pub dirty: bool,
    pub inv: [(u32, u32); ROWLENGTH as usize]
}

#[derive(Clone)]
pub struct ChestInventory {
    pub dirty: bool,
    pub inv: [(u32, u32); ROWLENGTH as usize * 4]
}