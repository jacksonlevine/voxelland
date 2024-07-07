use once_cell::sync::Lazy;

pub type Recipe = (Vec<(u32, u32)>, (u32, u32), bool);



pub static RECIPES: Lazy<[Recipe; 12]> = Lazy::new(|| [
    (vec![(6, 1)], (10, 4), true),
    (vec![(10, 4)], (19, 2), false),
    (vec![(10, 2)], (20, 10), false),
    (vec![(10, 8)], (21, 1), false),
    (vec![(11, 1)], (18, 1), true),
    (vec![(22, 1)], (30, 1), true),
    (vec![(10, 10)], (31, 1), true),
    (vec![(1, 1)], (8, 1), false),
    (vec![(18, 1), (23, 1)], (24, 1), true),
    (vec![(1, 1)], (14, 8), true),
    (vec![(5, 1)], (9, 1), false),
    (vec![(22, 1)], (33, 4), true),
]);