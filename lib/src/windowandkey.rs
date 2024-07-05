use crate::{blockinfo::Blocks, game::{self, Game, CURRENT_AVAIL_RECIPES, DECIDEDSPORMP, SINGLEPLAYER}, shader::Shader, text::Text, texture::Texture};
use glam::{Vec2, Vec4};
use glfw::{ffi::{glfwGetWindowMonitor, glfwSetWindowMonitor}, Action, Context, Glfw, GlfwReceiver, Key, Monitor, PWindow, Window, WindowEvent};
use once_cell::sync::Lazy;
use sys::ImVector_ImVec4;
use std::{path::Path, ptr::addr_of_mut, sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, time::{Duration, Instant}};
use imgui::*;
use imgui_opengl_renderer::Renderer;

pub static mut WINDOWWIDTH: i32 = 0;
pub static mut WINDOWHEIGHT: i32 = 0;




pub struct WindowAndKeyContext {
    pub width: u32,
    pub height: u32,
    pub game: Option<Game>,

    pub previous_time: Instant,
    pub delta_time: f32,

    pub glfw: Glfw,
    pub window: Arc<RwLock<PWindow>>,
    pub events: GlfwReceiver<(f64, WindowEvent)>,

    pub imgui: imgui::Context,
    pub guirenderer: imgui_opengl_renderer::Renderer,
    pub addressentered: Arc<AtomicBool>,
    pub serveraddress: Arc<Mutex<Option<String>>>,

    pub serveraddrbuffer: String
}

fn toggle_fullscreen(window_ptr: *mut glfw::ffi::GLFWwindow) {
    unsafe {
        let monitor = glfw::ffi::glfwGetWindowMonitor(window_ptr);
        if monitor.is_null() {
            let primary_monitor = glfw::ffi::glfwGetPrimaryMonitor();
            if !primary_monitor.is_null() {
                let mode = glfw::ffi::glfwGetVideoMode(primary_monitor);
                if !mode.is_null() {
                    glfw::ffi::glfwSetWindowMonitor(
                        window_ptr,
                        primary_monitor,
                        0,
                        0,
                        (*mode).width as i32,
                        (*mode).height as i32,
                        glfw::ffi::DONT_CARE,
                    );
                }
            }
        } else {
            glfw::ffi::glfwSetWindowMonitor(
                window_ptr,
                std::ptr::null_mut(),
                100,
                100,
                1280,
                720,
                glfw::ffi::DONT_CARE,
            );
        }
    }
}

impl WindowAndKeyContext {
    pub fn new(windowname: &'static str, width: u32, height: u32) -> Self {

        unsafe {
            WINDOWHEIGHT = height as i32;
            WINDOWWIDTH = width as i32;
        }
        
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        let (mut window, events) = glfw
            .create_window(width, height, windowname, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_scroll_polling(true);
        window.set_char_polling(true);
        window.make_current();

        // Initialize ImGui
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);


        let font_size = 16.0;
        imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../../font.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                oversample_h: 4,
                oversample_v: 4,
                pixel_snap_h: true,
                ..Default::default()
            }),
        }]);
        let mut renderer = Renderer::new(&mut imgui, |s| window.get_proc_address(s) as *const _);


        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CW);
        }

   

        let mut wak = WindowAndKeyContext {
            width,
            height,
            game: None,
            previous_time: Instant::now(),
            delta_time: 0.0,
            glfw,
            window: Arc::new(RwLock::new(window)),
            events,
            imgui,
            guirenderer: renderer,
            addressentered: Arc::new(AtomicBool::new(false)),
            serveraddress: Arc::new(Mutex::new(None)),
            serveraddrbuffer: String::with_capacity(128)
        };

        wak
    }

    

    pub fn run(&mut self) {
        self.glfw.poll_events();

        let current_time = Instant::now();
        self.delta_time = current_time
            .duration_since(self.previous_time)
            .as_secs_f32();
        self.previous_time = current_time;


        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        let mut main_menu = false;

        unsafe {
            match DECIDEDSPORMP {
                false => {

                    self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(self.delta_time));
            
                    let (width, height) = self.window.read().unwrap().get_framebuffer_size();
                    self.imgui.io_mut().display_size = [width as f32, height as f32];
                    
                    // Start the ImGui frame
                    let ui = self.imgui.frame();
        
                    let window_flags = WindowFlags::NO_DECORATION
                        | WindowFlags::NO_MOVE
                        | WindowFlags::NO_RESIZE
                        | WindowFlags::NO_SCROLLBAR
                        | WindowFlags::NO_TITLE_BAR
                        | WindowFlags::NO_BACKGROUND;
        
                    let window_size = (700.0, 700.0);
        
                    let window_pos = [width as f32 / 2.0 - (window_size.0/2.0), height as f32 / 2.0 - (window_size.1/2.0)];
        
                    ui.window("Transparent Window")
                        .size([window_size.0, window_size.1], Condition::Always)
                        .position(window_pos, Condition::Always)
                        .flags(window_flags)
                        .build(|| {
                            let button_width = 500.0;
                            let button_height = 20.0;
                            let window_size = ui.window_size();
        
                            let available_width = window_size[0];
                            let available_height = window_size[1];
        
                            let pos_x = (available_width - button_width) / 2.0;
                            let mut pos_y = (available_height - (button_height) - 10.0 ) / 2.0;

                                ui.set_cursor_pos([pos_x, pos_y - 50.0]);

                                ui.text_colored([1.0, 0.0, 0.0, 1.0], "Welcome! Please choose an option.");
        
                                ui.set_cursor_pos([pos_x, pos_y - 25.0]);

                                
        
                                if ui.button_with_size("Singleplayer", [button_width, button_height]) {
                                    unsafe {
                                        SINGLEPLAYER = true;
                                        DECIDEDSPORMP = true;
                                    }
                                }
        
                                ui.set_cursor_pos([pos_x, pos_y]);
        
                                
        
                                if ui.button_with_size("Multiplayer", [button_width, button_height]) {
                                    unsafe {
                                        SINGLEPLAYER = false;
                                        DECIDEDSPORMP = true;
                                    }
                                }
                        });
        
                    // Render the ImGui frame
                    self.guirenderer.render(&mut self.imgui);
        
        
        
                    let io = self.imgui.io_mut();
                    for (_, event) in glfw::flush_messages(&self.events) {
        
                        
        
                        match event {
                            glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                                let index = match mousebutton {
                                    glfw::MouseButton::Button1 => 0,
                                    glfw::MouseButton::Button2 => 1,
                                    glfw::MouseButton::Button3 => 2,
                                    glfw::MouseButton::Button4 => 3,
                                    glfw::MouseButton::Button5 => 4,
                                    glfw::MouseButton::Button6 => 5,
                                    glfw::MouseButton::Button7 => 6,
                                    glfw::MouseButton::Button8 => 7,
                                    _ => return,
                                };
                                io.mouse_down[index] = action == glfw::Action::Press;
                                    
                            }
                            glfw::WindowEvent::FramebufferSize(wid, hei) => {
                                self.width = wid as u32;
                                self.height = hei as u32;
                                unsafe {
                                    gl::Viewport(0, 0, wid, hei);
                                }
                            }
                            glfw::WindowEvent::CursorPos(xpos, ypos) => {
        
                                    io.mouse_pos = [xpos as f32, ypos as f32];
        
                                
                            }
                            glfw::WindowEvent::Key(key, scancode, action, _modifiers) => {
        
                                let pressed = action == glfw::Action::Press || action == glfw::Action::Repeat;
                                io.keys_down[key as usize] = pressed;
        
                            if action == glfw::Action::Press {
                                match key {
                                    glfw::Key::LeftShift | glfw::Key::RightShift => io.key_shift = true,
                                    glfw::Key::LeftControl | glfw::Key::RightControl => io.key_ctrl = true,
                                    glfw::Key::LeftAlt | glfw::Key::RightAlt => io.key_alt = true,
                                    glfw::Key::LeftSuper | glfw::Key::RightSuper => io.key_super = true,
                                    glfw::Key::Backspace => {
                                        io.keys_down[glfw::Key::Backspace as usize] = true;
                                        io.add_input_character('\u{8}');
                                    }
                                    _ => {}
                                }
                            } else if action == glfw::Action::Release {
                                match key {
                                    glfw::Key::LeftShift | glfw::Key::RightShift => io.key_shift = false,
                                    glfw::Key::LeftControl | glfw::Key::RightControl => io.key_ctrl = false,
                                    glfw::Key::LeftAlt | glfw::Key::RightAlt => io.key_alt = false,
                                    glfw::Key::LeftSuper | glfw::Key::RightSuper => io.key_super = false,
                                    glfw::Key::Backspace => io.keys_down[glfw::Key::Backspace as usize] = false,
                                    _ => {}
                                }
                            }
        
        
                            }   
                            glfw::WindowEvent::Char(char) => {
                                io.add_input_character(char);
                            }
                            glfw::WindowEvent::Scroll(x, y) => {
                                io.mouse_wheel_h += x as f32;
                                io.mouse_wheel += y as f32;
        
                            }
                            _ => {}
                        }
                    }
        

                }
                true => {
                    match self.game.as_mut() {
                        Some(g) => {
            
                            let gmenuopen = g.vars.menu_open;
            
                            let gcraftopen = g.crafting_open;
            
                            if g.vars.main_menu {
                                main_menu = true;
                            } else {
            
                                if g.loadedworld.load(std::sync::atomic::Ordering::Relaxed) {
            
                                    
                                    g.update();
            
                                    
            
                                    
            
            
                                }
            
                                if gmenuopen {
            
                                    let cb = g.currentbuttons.clone();
            
                                    self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(self.delta_time));
            
                                    let (width, height) = self.window.read().unwrap().get_framebuffer_size();
                                    self.imgui.io_mut().display_size = [width as f32, height as f32];
                                    
                                    // Start the ImGui frame
                                    let ui = self.imgui.frame();
            
                                    let window_flags = WindowFlags::NO_DECORATION
                                        | WindowFlags::NO_MOVE
                                        | WindowFlags::NO_RESIZE
                                        | WindowFlags::NO_SCROLLBAR
                                        | WindowFlags::NO_TITLE_BAR
                                        | WindowFlags::NO_BACKGROUND;
            
                                    let window_size = (700.0, 700.0);
            
                                    let window_pos = [width as f32 / 2.0 - (window_size.0/2.0), height as f32 / 2.0 - (window_size.1/2.0)];
            
                                    ui.window("Transparent Window")
                                        .size([window_size.0, window_size.1], Condition::Always)
                                        .position(window_pos, Condition::Always)
                                        .flags(window_flags)
                                        .build(|| {
                                            let button_width = 200.0;
                                            let button_height = 20.0;
                                            let window_size = ui.window_size();
            
                                            let available_width = window_size[0];
                                            let available_height = window_size[1];
            
                                            let pos_x = (available_width - button_width) / 2.0;
                                            let mut pos_y = (available_height - (cb.len() as f32 * button_height) - 10.0 * (cb.len() as f32 - 1.0)) / 2.0;
            
                                            for (buttonname, command) in cb {
                                                ui.set_cursor_pos([pos_x, pos_y]);
                                                if ui.button_with_size(buttonname, [button_width, button_height]) {
                                                    g.button_command(command);
                                                }
                                                pos_y += button_height + 10.0; // Add some spacing between buttons
                                            }
                                        });
            
                                    // Render the ImGui frame
                                    self.guirenderer.render(&mut self.imgui);
                                } else {
                                    if gcraftopen {
            
            
            
            
                                        let cb = g.currentbuttons.clone();
            
                                        self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(self.delta_time));
            
                                        let (width, height) = self.window.read().unwrap().get_framebuffer_size();
                                        self.imgui.io_mut().display_size = [width as f32, height as f32];
                                        
                                        // Start the ImGui frame
                                        let ui = self.imgui.frame();
            
                                        let window_flags = WindowFlags::NO_DECORATION
                                            | WindowFlags::NO_MOVE
                                            | WindowFlags::NO_RESIZE
                                            | WindowFlags::NO_TITLE_BAR;
            
                                        let window_size = (700.0, 700.0);
            
                                        let window_pos = [width as f32 / 2.0 - (window_size.0/2.0), height as f32 / 2.0 - (window_size.1/2.0)];
            
                                        ui.window("Transparent Window")
                                            .size([window_size.0, window_size.1], Condition::Always)
                                            .position(window_pos, Condition::Always)
                                            .flags(window_flags)
                                            .build(|| {
                                                let button_width = 200.0;
                                                let button_height = 20.0;
                                                let window_size = ui.window_size();
            
                                                let available_width = window_size[0];
                                                let available_height = window_size[1];
            
                                                let pos_x = (available_width - button_width) / 2.0;
                                                let mut pos_y = (available_height - (cb.len() as f32 * button_height) - 10.0 * (cb.len() as f32 - 1.0)) / 2.0;
            
                                                // for (buttonname, command) in cb {
                                                //     ui.set_cursor_pos([pos_x, pos_y]);
                                                //     if ui.button_with_size(buttonname, [button_width, button_height]) {
                                                //         g.button_command(command);
                                                //     }
                                                //     pos_y += button_height + 10.0; // Add some spacing between buttons
                                                // }
                                                unsafe {
                                                    for (index, recipe) in CURRENT_AVAIL_RECIPES.iter().enumerate() {
                                                        ui.set_cursor_pos([pos_x, pos_y]);
                                                        let str = format!("{}, {}", Blocks::get_name(recipe.1.0), recipe.1.1);
                                                        if ui.button_with_size(str, [button_width, button_height]) {
                                                            g.craft_recipe_index(index);
                                                        }
            
                                                        let mut costs = String::from("Using ");
            
                                                        for (index, entry) in recipe.0.iter().enumerate() {
                                                            costs += entry.1.to_string().as_str();
                                                            costs += " ";
                                                            costs += Blocks::get_name(entry.0);
                                                            if index < (recipe.0.len()-1) {
                                                                costs += ", ";
                                                            } else {
                                                                costs += ".";
                                                            }
                                                        }
            
            
                                                        ui.text_colored([1.0, 0.0, 0.0, 1.0], costs);
                                                        g.update_inventory();
                                                        Game::update_avail_recipes(&g.inventory.clone());
                                                        
            
                                                        pos_y += button_height + 10.0; // Add some spacing between buttons
                                                    }
                                                }
                                                
                                                
                                            });
            
            
            
                                            // Render the ImGui frame
                                            self.guirenderer.render(&mut self.imgui);
            
            
                                    }
                                }
            
                                
            
                                
                                let io = self.imgui.io_mut();
                                for (_, event) in glfw::flush_messages(&self.events) {
            
                                    
            
                                    match event {
                                        
                                        glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                                            let index = match mousebutton {
                                                glfw::MouseButton::Button1 => 0,
                                                glfw::MouseButton::Button2 => 1,
                                                glfw::MouseButton::Button3 => 2,
                                                glfw::MouseButton::Button4 => 3,
                                                glfw::MouseButton::Button5 => 4,
                                                glfw::MouseButton::Button6 => 5,
                                                glfw::MouseButton::Button7 => 6,
                                                glfw::MouseButton::Button8 => 7,
                                                _ => return,
                                            };
                                            io.mouse_down[index] = action == glfw::Action::Press;
            
                                            if !io.want_capture_mouse || !gmenuopen {
                                                if mousebutton == glfw::MouseButtonLeft {
                                                    
                                                    if !io.want_capture_mouse {
                                                        
                                                        if !gmenuopen {
                                                            self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Disabled);
                                                            self.game.as_mut().unwrap().set_mouse_focused(true);
                                                        }
                                                        
                                                    }
                                                    
                                                }
                                                self.game
                                                    .as_mut()
                                                    .unwrap()
                                                    .mouse_button(mousebutton, action);
                                            }
                                                
                                        }
                                        glfw::WindowEvent::FramebufferSize(wid, hei) => {
                                            self.width = wid as u32;
                                            self.height = hei as u32;
                                            unsafe {
                                                gl::Viewport(0, 0, wid, hei);
                                                WINDOWHEIGHT = hei;
                                                WINDOWWIDTH = wid
                                            }
                                        }
                                        glfw::WindowEvent::CursorPos(xpos, ypos) => {
                                            let g = self.game.as_mut().unwrap();
                                            g.cursor_pos(xpos, ypos);
                                            if !g.vars.mouse_focused {
                                                io.mouse_pos = [xpos as f32, ypos as f32];
                                            }
                                            
                                        }
                                        glfw::WindowEvent::Key(key, scancode, action, _modifiers) => {
            
                                            let pressed = action == glfw::Action::Press || action == glfw::Action::Repeat;
                                            io.keys_down[scancode as usize] = pressed;
            
                                            if (!io.want_capture_keyboard && !io.want_text_input  ) || gmenuopen {
                                                
                                                self.game.as_mut().unwrap().keyboard(key, action);
            
                                                if key == Key::Escape {
                                                    if self.game.as_mut().unwrap().vars.menu_open {
                                                    
                                                        self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Normal);
                                                        self.game.as_mut().unwrap().set_mouse_focused(false);
                                                    } else {
                                                        self.game.as_mut().unwrap().vars.menu_open = false;
                                                        self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Disabled);
                                                        self.game.as_mut().unwrap().set_mouse_focused(true);
                                                    }
                                                    
                                                }
            
                                            }
            
                                            match key {
                                                Key::F11 => {
                                                    if action == Action::Press {
                                                        let wind = self.window.write().unwrap();
                                                        toggle_fullscreen(wind.window_ptr())
                                                        
                                                    }
                                                }
                                                _ => {
            
                                                }
                                            }
                                            
                                        }
                                        glfw::WindowEvent::Scroll(x, y) => {
                                            io.mouse_wheel_h += x as f32;
                                            io.mouse_wheel += y as f32;
            
                                            self.game.as_mut().unwrap().scroll(y);
                                        }
                                        _ => {}
                                    }
                                }
            
            
            
                            }
                        }
                        None => {
            
            
            
                                main_menu = true;
            
            
            
            
                        }
                    }
            
                    if main_menu && !SINGLEPLAYER {
                        
            
            
            
            
            
            
                            
            
                        self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(self.delta_time));
            
                        let (width, height) = self.window.read().unwrap().get_framebuffer_size();
                        self.imgui.io_mut().display_size = [width as f32, height as f32];
                        
                        // Start the ImGui frame
                        let ui = self.imgui.frame();
            
                        let window_flags = WindowFlags::NO_DECORATION
                            | WindowFlags::NO_MOVE
                            | WindowFlags::NO_RESIZE
                            | WindowFlags::NO_SCROLLBAR
                            | WindowFlags::NO_TITLE_BAR
                            | WindowFlags::NO_BACKGROUND;
            
                        let window_size = (700.0, 700.0);
            
                        let window_pos = [width as f32 / 2.0 - (window_size.0/2.0), height as f32 / 2.0 - (window_size.1/2.0)];
            
                        ui.window("Transparent Window")
                            .size([window_size.0, window_size.1], Condition::Always)
                            .position(window_pos, Condition::Always)
                            .flags(window_flags)
                            .build(|| {
                                let button_width = 500.0;
                                let button_height = 20.0;
                                let window_size = ui.window_size();
            
                                let available_width = window_size[0];
                                let available_height = window_size[1];
            
                                let pos_x = (available_width - button_width) / 2.0;
                                let mut pos_y = (available_height - (button_height) - 10.0 ) / 2.0;
            
            
                                    ui.set_cursor_pos([pos_x, pos_y]);
            
                                    
            
                                    if ui.button_with_size("Enter server address:", [button_width, button_height]) {
            
                                    }
            
                                    ui.set_cursor_pos([pos_x, pos_y + 25.0]);
                                    
                                    ui.input_text("##serveraddress", &mut self.serveraddrbuffer)
                                    .flags(InputTextFlags::ALWAYS_OVERWRITE)
                                    
                                    .build();
            
                                    ui.set_cursor_pos([pos_x, pos_y + 50.0]);
            
            
                                    if ui.button_with_size("Connect", [button_width, button_height]) {
                                        unsafe {
                                            SINGLEPLAYER = false;
                                            DECIDEDSPORMP = true;
                                        }
                                        
                                        *(self.serveraddress.lock().unwrap()) = Some(self.serveraddrbuffer.clone());
                                        self.addressentered.store(true, std::sync::atomic::Ordering::Relaxed);
                                    }
                                    pos_y += button_height + 10.0; // Add some spacing between buttons
            
                            });
            
                        // Render the ImGui frame
                        self.guirenderer.render(&mut self.imgui);
            
            
            
                        let io = self.imgui.io_mut();
                        for (_, event) in glfw::flush_messages(&self.events) {
            
                            
            
                            match event {
                                glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                                    let index = match mousebutton {
                                        glfw::MouseButton::Button1 => 0,
                                        glfw::MouseButton::Button2 => 1,
                                        glfw::MouseButton::Button3 => 2,
                                        glfw::MouseButton::Button4 => 3,
                                        glfw::MouseButton::Button5 => 4,
                                        glfw::MouseButton::Button6 => 5,
                                        glfw::MouseButton::Button7 => 6,
                                        glfw::MouseButton::Button8 => 7,
                                        _ => return,
                                    };
                                    io.mouse_down[index] = action == glfw::Action::Press;
                                        
                                }
                                glfw::WindowEvent::FramebufferSize(wid, hei) => {
                                    self.width = wid as u32;
                                    self.height = hei as u32;
                                    unsafe {
                                        gl::Viewport(0, 0, wid, hei);
                                    }
                                }
                                glfw::WindowEvent::CursorPos(xpos, ypos) => {
            
                                        io.mouse_pos = [xpos as f32, ypos as f32];
            
                                    
                                }
                                glfw::WindowEvent::Key(key, scancode, action, _modifiers) => {
            
                                    let pressed = action == glfw::Action::Press || action == glfw::Action::Repeat;
                                    io.keys_down[key as usize] = pressed;
            
                                if action == glfw::Action::Press {
                                    match key {
                                        glfw::Key::LeftShift | glfw::Key::RightShift => io.key_shift = true,
                                        glfw::Key::LeftControl | glfw::Key::RightControl => io.key_ctrl = true,
                                        glfw::Key::LeftAlt | glfw::Key::RightAlt => io.key_alt = true,
                                        glfw::Key::LeftSuper | glfw::Key::RightSuper => io.key_super = true,
                                        glfw::Key::Backspace => {
                                            io.keys_down[glfw::Key::Backspace as usize] = true;
                                            io.add_input_character('\u{8}');
                                        }
                                        _ => {}
                                    }
                                } else if action == glfw::Action::Release {
                                    match key {
                                        glfw::Key::LeftShift | glfw::Key::RightShift => io.key_shift = false,
                                        glfw::Key::LeftControl | glfw::Key::RightControl => io.key_ctrl = false,
                                        glfw::Key::LeftAlt | glfw::Key::RightAlt => io.key_alt = false,
                                        glfw::Key::LeftSuper | glfw::Key::RightSuper => io.key_super = false,
                                        glfw::Key::Backspace => io.keys_down[glfw::Key::Backspace as usize] = false,
                                        _ => {}
                                    }
                                }
            
            
                                }   
                                glfw::WindowEvent::Char(char) => {
                                    io.add_input_character(char);
                                }
                                glfw::WindowEvent::Scroll(x, y) => {
                                    io.mouse_wheel_h += x as f32;
                                    io.mouse_wheel += y as f32;
            
                                }
                                _ => {}
                            }
                        }
            
            
                            
                    }
            
                    
                }
            }
        }

        

        self.window.write().unwrap().swap_buffers();
    }
}
