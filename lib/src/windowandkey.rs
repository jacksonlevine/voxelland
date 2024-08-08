use crate::{blockinfo::Blocks, game::{Game, CURRENT_AVAIL_RECIPES, DECIDEDSPORMP, MOUSEX, MOUSEY, SHOWTOOLTIP, SINGLEPLAYER, TOOLTIPNAME}, recipes::{RecipeEntry, RECIPES_DISABLED, RECIPE_COOLDOWN_TIMER}, statics::{LAST_ENTERED_SERVERADDRESS, LOAD_OR_INITIALIZE_STATICS, SAVE_LESA}, texture::Texture};

use glfw::{Action, Context, Glfw, GlfwReceiver, Key, Modifiers, PWindow, WindowEvent};
use once_cell::sync::Lazy;


use std::{sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, time::{Duration, Instant}};
use imgui::*;
use imgui::{Key as ImGuiKey};
use imgui_opengl_renderer::Renderer;

pub static mut WINDOWWIDTH: i32 = 0;
pub static mut WINDOWHEIGHT: i32 = 0;

pub static mut uncapkb: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));



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

    pub serveraddrbuffer: String,

    pub logo: Texture
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

        let scale_factor = 1.0;


        imgui.io_mut().font_global_scale = scale_factor;
        imgui.style_mut().scale_all_sizes(scale_factor);

        {
            let mut io = imgui.io_mut();

            io.key_map[ImGuiKey::Backspace as usize] = Key::Backspace as u32;
            io.key_map[ImGuiKey::Tab as usize] = Key::Tab as u32;
            io.key_map[ImGuiKey::LeftArrow as usize] = Key::Left as u32;
            io.key_map[ImGuiKey::RightArrow as usize] = Key::Right as u32;
            io.key_map[ImGuiKey::UpArrow as usize] = Key::Up as u32;
            io.key_map[ImGuiKey::DownArrow as usize] = Key::Down as u32;
            io.key_map[ImGuiKey::PageUp as usize] = Key::PageUp as u32;
            io.key_map[ImGuiKey::PageDown as usize] = Key::PageDown as u32;
            io.key_map[ImGuiKey::Home as usize] = Key::Home as u32;
            io.key_map[ImGuiKey::End as usize] = Key::End as u32;
            io.key_map[ImGuiKey::Insert as usize] = Key::Insert as u32;
            io.key_map[ImGuiKey::Delete as usize] = Key::Delete as u32;
            io.key_map[ImGuiKey::Backspace as usize] = Key::Backspace as u32;
            io.key_map[ImGuiKey::Space as usize] = Key::Space as u32;
            io.key_map[ImGuiKey::Enter as usize] = Key::Enter as u32;
            io.key_map[ImGuiKey::Escape as usize] = Key::Escape as u32;
        }


        let font_size = 16.0;
        imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../../font.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                oversample_h: 1,
                oversample_v: 1,
                pixel_snap_h: true,
                size_pixels: 72.0,
                ..Default::default()
            }),
        }]);

        
        let renderer = Renderer::new(&mut imgui, |s| window.get_proc_address(s) as *const _);


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
            serveraddrbuffer: String::with_capacity(128),
            logo: Texture::new("assets/Untitled3.png").unwrap()
        };

        LOAD_OR_INITIALIZE_STATICS();
        unsafe {
            wak.serveraddrbuffer = (*LAST_ENTERED_SERVERADDRESS).clone();
            wak.serveraddrbuffer.reserve(100);
        }
        

        wak
    }

    

    pub fn run(&mut self) {
        
        #[cfg(feature = "glfw")]
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
                    let window_pos = [width as f32 / 2.0 - (window_size.0 / 2.0), height as f32 / 2.0 - (window_size.1 / 2.0)];

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
                            let pos_y = (available_height - (button_height) - 10.0) / 2.0;

                            ui.set_cursor_pos([pos_x, pos_y - 240.0]);

                            // Scale down the image to 50% of its original size
                            let scaled_size = [
                                (self.logo.size.0 as f32 * 0.5).round(),
                                (self.logo.size.1 as f32 * 0.5).round(),
                            ];

                            // Calculate the position to center the image
                            let image_pos_x = (available_width - scaled_size[0]) / 2.0;
                            let image_pos_y = ((available_height - scaled_size[1]) / 2.0) - 150.0;

                            ui.set_cursor_pos([image_pos_x, image_pos_y]);

                            let texture_id = imgui::TextureId::from(self.logo.id as usize);
                            imgui::Image::new(texture_id, scaled_size).build(&ui);

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
                            glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
        
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

                            #[cfg(feature = "glfw")]
                            let gchestopen = g.hud.chest_open;

                            #[cfg(not(feature = "glfw"))]
                            let gchestopen = false;
                            
            
                            if g.vars.main_menu {
                                main_menu = true;
                            } else {
            
                                if g.loadedworld.load(std::sync::atomic::Ordering::Relaxed) {
            
                                    
                                    g.update();
                                    
                                    let state = self.glfw.get_joystick(glfw::JoystickId::Joystick1);

                                    static mut lastx: f64 = 0.0;
                                    static mut lasty: f64 = 0.0;

                                    static mut x: f64 = 0.0;
                                    static mut y: f64 = 0.0;

                                    
                                    let axes = state.get_axes();

                                    if axes.len() >= 2 {
                                        unsafe {
                                            x += axes[0] as f64;
                                            y += axes[1] as f64;
    
                                            if lastx != x || lasty != y {
                                                lastx = x;
                                                lasty = y;
                                                g.cursor_pos(x, y);
                                            }
    
                                            
                                        }
                                    }
                                    
                                    
            
                                    
            
            
                                }

                                self.imgui.io_mut().update_delta_time(Duration::from_secs_f32(self.delta_time));

                                if uncapkb.load(std::sync::atomic::Ordering::Relaxed) {
                                    self.imgui.io_mut().want_capture_keyboard = false;
                                    self.imgui.io_mut().want_text_input = false;
                                    self.imgui.io_mut().want_capture_mouse = false;
                                    unsafe {
                                        uncapkb.store(false, std::sync::atomic::Ordering::Relaxed);
                                    } 
                                }

                                if gchestopen {

                                    let ui = self.imgui.frame();

                                    let window_flags = WindowFlags::NO_DECORATION
                                        | WindowFlags::NO_MOVE
                                        | WindowFlags::NO_RESIZE
                                        | WindowFlags::NO_SCROLLBAR
                                        | WindowFlags::NO_TITLE_BAR
                                        | WindowFlags::NO_INPUTS;
                                    
                                    if SHOWTOOLTIP {
                                        ui.window("Mouse Tooltip Window")
                                        .size([300.0, 50.0], Condition::Always)
                                        .position([MOUSEX as f32, MOUSEY as f32], Condition::Always)
                                        .flags(window_flags)
                                        .build(|| {
                                            ui.text(TOOLTIPNAME);
                                        });
                                    }

                                    self.guirenderer.render(&mut self.imgui);
                                }
            
                                if gmenuopen {
            
                                    let gamecurrentbuttons = g.currentbuttons.clone();
                                    
                                    
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
                                    
                                    
                                    // unsafe {
                                    //     uncapkb.store(false, std::sync::atomic::Ordering::Relaxed);
                                    // } 

                          

                                    ui.window("Transparent Window")
                                        .size([window_size.0, window_size.1], Condition::Always)
                                        .position(window_pos, Condition::Always)
                                        .flags(window_flags)
                                        .build(|| {

                                            let len = gamecurrentbuttons.len();

                                            let button_height = 20.0;
                                            let window_size = ui.window_size();
                                            let available_width = window_size[0];
                                            let available_height = window_size[1] + 200.0;
                                            let mut pos_y = (available_height - (len as f32 * button_height) - 10.0 * (len as f32 - 1.0)) / 2.0;
            


                                            
                                            for (buttonname, command) in gamecurrentbuttons {






                                                let button_width = buttonname.len() as f32 * 20.0;
                                            
                                            
            
                                            
            
                                            let pos_x = (available_width - button_width) / 2.0;


                                                ui.set_cursor_pos([pos_x, pos_y]);
                                                if buttonname.starts_with("Slider") {
                                                    let truncated_name = buttonname.split_at(6).1;
                                                    if ui.slider(truncated_name, 0.1, 3.0, &mut g.vars.sensitivity) {
                                                        //g.button_command(command);
                                                    }
                                                } else {
                                                    if ui.button_with_size(buttonname, [button_width, button_height]) {
                                                        g.button_command(command);
                                                        unsafe {
                                                            uncapkb.store(true, std::sync::atomic::Ordering::Relaxed);
                                                        } 
                                                    }
                                                }
                                                
                                                pos_y += button_height + 10.0; 
                                            }
                                        });

                                        
            
                                    // Render the ImGui frame
                                    self.guirenderer.render(&mut self.imgui);
                                } else {
                                    if gcraftopen {
            
            
            
            
                                        let cb = g.currentbuttons.clone();
            
                                        
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
                                        let mut recipeindexscrafted = Vec::new();

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


                                                    for (index, recipeent) in CURRENT_AVAIL_RECIPES.lock().unwrap().iter_mut().enumerate() {
      
                                                        let recipe = recipeent.recipe.clone();
                                                        ui.set_cursor_pos([pos_x, pos_y]);
                                                        let str = format!("{}, {}", Blocks::get_name(recipe.1.0), recipe.1.1);
                                                        if RECIPES_DISABLED {
                                                            ui.text_colored([0.0, 0.0, 1.0, 1.0], str);
                                                           
                
                                                        } else {
                                                            if ui.button_with_size(str, [button_width, button_height]) {
                                                                recipeindexscrafted.push(index);
                                                                //g.craft_recipe_index(index);
                                                                recipeent.disabled = true;
                                                                unsafe {
                                                                    RECIPES_DISABLED = true;
                                                                }
                                                            }
                
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
                                                        
                                                        
                                                        
            
                                                        pos_y += button_height + 10.0; // Add some spacing between buttons
                                                    }
                                                }

                                                
                                                
                                                
                                            });

                                            

                                            for recipe in recipeindexscrafted {
                                                g.craft_recipe_index(recipe);
                                            }
                                            Game::update_avail_recipes(&g.inventory.clone());

                                            unsafe {
                                                if RECIPES_DISABLED {
                                                    if RECIPE_COOLDOWN_TIMER < 0.5 {
                                                        RECIPE_COOLDOWN_TIMER += self.delta_time;
                                                    } else {
                                                        RECIPES_DISABLED = false;
                                                        RECIPE_COOLDOWN_TIMER = 0.0;
                                                    }

                                                }
                                                
                                            }

                                            #[cfg(feature = "glfw")]
                                            g.update_inventory();


            
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

                                            // println!("Got a m.o.u.s.e. event");
                                            //         println!("io.want_capture_mouse: {}, gmenuopen: {}", 
                                            //         io.want_capture_mouse,
                                            //         gmenuopen);
            
                                            if !io.want_capture_mouse && !gmenuopen {
                                                if mousebutton == glfw::MouseButtonLeft {

                                                    

                                                    
                                                    if !io.want_capture_mouse {
                                                        
                                                        
                                                        if !gmenuopen && !gchestopen {
                                                            self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Disabled);
                                                            self.game.as_mut().unwrap().set_mouse_focused(true);
                                                        }
                                                        
                                                    }
                                                    
                                                }
                                                #[cfg(feature = "glfw")]
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
                                            // println!("Got a kb event");
                                            // println!("io.want_capture_keyboard: {}, io.want_text_input: {}, gmenuopen: {}", 
                                            // io.want_capture_keyboard,
                                            // io.want_text_input,
                                            // gmenuopen);

                                            if (!io.want_capture_keyboard && !io.want_text_input  ) && !gmenuopen {
                                                
                                                #[cfg(feature = "glfw")]
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
            
                                            } else {
                                                //println!()
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
                                            
                                            if !gmenuopen {
                                                #[cfg(feature = "glfw")]
                                                self.game.as_mut().unwrap().scroll(y);
                                            }
                                            
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
                                        unsafe {
                                            *LAST_ENTERED_SERVERADDRESS = self.serveraddrbuffer.clone();
                                        }
                                        SAVE_LESA();
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
                                glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
                             
                                    let pressed = action == glfw::Action::Press || action == glfw::Action::Repeat;

                                    if (key as usize) < 512 {
                                        io.keys_down[key as usize] = pressed;

                                        Self::set_mod(io, modifiers);
                                        io.keys_down[key as usize] = action != Action::Release;
                                    }
                                    

            
                                if action == glfw::Action::Press {
                                    match key {
                                        glfw::Key::LeftShift | glfw::Key::RightShift => io.key_shift = true,
                                        glfw::Key::LeftControl | glfw::Key::RightControl => io.key_ctrl = true,
                                        glfw::Key::LeftAlt | glfw::Key::RightAlt => io.key_alt = true,
                                        glfw::Key::LeftSuper | glfw::Key::RightSuper => io.key_super = true,
                                        glfw::Key::Backspace => {
                                            io.keys_down[glfw::Key::Backspace as usize] = true;
                                            io.add_input_character('\x08');
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

    fn set_mod(io: &mut imgui::Io, modifier: Modifiers) {
        io.key_ctrl = modifier.intersects(Modifiers::Control);
        io.key_alt = modifier.intersects(Modifiers::Alt);
        io.key_shift = modifier.intersects(Modifiers::Shift);
        io.key_super = modifier.intersects(Modifiers::Super);
    }
}
