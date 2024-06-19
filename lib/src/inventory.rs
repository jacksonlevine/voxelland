pub struct Inventory {
    pub dirty: bool,
    pub inv: [(u32, u32); 5]
}

pub struct ChestInventory {
    pub dirty: bool,
    pub inv: [(u32, u32); 20]
}