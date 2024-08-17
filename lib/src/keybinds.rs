use std::collections::HashMap;


use once_cell::sync::Lazy;

pub enum Rebindable {
    Key(i32),
    MouseButton(glfw::MouseButton)
}


pub struct AboutToRebind {
    pub key: Rebindable,
    pub action: String
}

pub static mut LISTENINGFORREBIND: bool = false;

pub static mut ABOUTTOREBIND: Option<AboutToRebind> = None;


