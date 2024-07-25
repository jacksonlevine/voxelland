use once_cell::sync::Lazy;
use tracing::info;
pub type Recipe = (Vec<(u32, u32)>, (u32, u32), bool);

pub static mut RECIPES_DISABLED: bool = false;
pub static mut RECIPE_COOLDOWN_TIMER: f32 = 0.0;

#[derive(Clone)]
pub struct RecipeEntry {
    pub recipe: Recipe,
    pub disabled: bool,
    pub disabledtimer: f32
}


impl RecipeEntry {
    #[inline]
    pub const fn from_recipe(recipe: Recipe) -> Self {
        Self {
            recipe,
            disabled: false,
            disabledtimer: 0.0
        }
    }

    pub fn tick_disabled_timer(&mut self, dt: f32) {
        if self.disabled {
            info!("I'm ticking disabled timer deta time: {dt}, my timer: {}", self.disabledtimer);
            if self.disabledtimer > 1.0 {
                self.disabledtimer = 0.0;
                self.disabled = false;
                
            } else {
                self.disabledtimer += dt;
            }
        }
    }
}



pub static RECIPES: Lazy<[Recipe; 21]> = Lazy::new(|| [
    (vec![(6, 1)], (10, 4), true),
    (vec![(10, 4)], (19, 2), false),
    (vec![(10, 2)], (20, 10), false),
    (vec![(10, 8)], (21, 1), false),
    (vec![(11, 1)], (18, 1), true),
    (vec![(22, 1)], (30, 1), true),
    (vec![(10, 10)], (31, 1), true),
    (vec![(1, 1)], (8, 1), false),
    (vec![(18, 1), (34, 1)], (24, 1), true),
    (vec![(1, 1)], (14, 8), true),
    (vec![(5, 1)], (9, 1), false),
    (vec![(22, 1)], (33, 4), true),
    (vec![(32, 1), (18, 1)], (27, 1), true),
    (vec![(24, 1), (27, 1)], (25, 1), true),
    (vec![(33, 1), (18, 1)], (26, 1), true),
    (vec![(10, 1), (35, 3)], (36, 1), true),

    (vec![(10, 1), (35, 17)], (37, 1), true),
    (vec![(10, 1), (35, 8)], (38, 1), true),
    (vec![(10, 1), (35, 14)], (39, 1), true),

    (vec![(24, 1), (26, 1)], (40, 1), true),
    (vec![(10, 1), (22, 1)], (42, 1), true)
]);