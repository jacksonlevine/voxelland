use once_cell::sync::Lazy;

pub type Recipe = (Vec<(u32, u32)>, (u32, u32));



pub static RECIPES: Lazy<[Recipe; 2]> = Lazy::new(|| [
    (vec![(6, 1)], (10, 4)),
    (vec![(10, 4)], (19, 2)),
]);