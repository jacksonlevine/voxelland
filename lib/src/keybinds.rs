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


pub static mut KEYBOARD_BINDINGS: Lazy<HashMap<i32, String>> = Lazy::new(|| 
    HashMap::from([
        (glfw::Key::Escape.get_scancode().unwrap(), "Exit/Menu".into()),
        (glfw::Key::W.get_scancode().unwrap(), "Forward".into()),
        (glfw::Key::A.get_scancode().unwrap(), "Left".into()),
        (glfw::Key::S.get_scancode().unwrap(), "Backward".into()),
        (glfw::Key::D.get_scancode().unwrap(), "Right".into()),

        (glfw::Key::C.get_scancode().unwrap(), "Craft".into()),
        
        (glfw::Key::Space.get_scancode().unwrap(), "Jump/Swim/Climb Up".into()),
        (glfw::Key::LeftShift.get_scancode().unwrap(), "Sprint".into()),
        (glfw::Key::LeftControl.get_scancode().unwrap(), "Crouch".into()),

        (glfw::Key::Num0.get_scancode().unwrap(), "Fov Up".into()),
        (glfw::Key::Num9.get_scancode().unwrap(), "Fov Down".into()),
    ])
);


pub static mut MOUSE_BINDINGS: Lazy<HashMap<glfw::MouseButton, String>> = Lazy::new(|| 
    HashMap::from([
        (glfw::MouseButtonRight, "Place/Use".into()),
        (glfw::MouseButtonLeft, "Break/Attack".into()),

    ])
);