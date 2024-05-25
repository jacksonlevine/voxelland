use crate::{game::Game, shader::Shader, text::Text, texture::Texture};
use glam::Vec2;
use glfw::{Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use std::{sync::{Arc, RwLock}, time::Instant};

pub struct WindowAndKeyContext {
    pub width: u32,
    pub height: u32,
    pub game: Option<Game>,

    pub previous_time: Instant,
    pub delta_time: f32,

    pub glfw: Glfw,
    pub window: Arc<RwLock<PWindow>>,
    pub events: GlfwReceiver<(f64, WindowEvent)>,

    
}

impl WindowAndKeyContext {
    pub fn new(windowname: &'static str, width: u32, height: u32) -> Self {
        
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
        window.make_current();

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
            events
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

        self.game.as_mut().unwrap().update();
        

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::MouseButton(mousebutton, action, _) => {
                    if mousebutton == glfw::MouseButtonLeft {
                        self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Disabled);
                        self.game.as_mut().unwrap().set_mouse_focused(true);
                    }
                    self.game
                        .as_mut()
                        .unwrap()
                        .mouse_button(mousebutton, action);
                }
                glfw::WindowEvent::FramebufferSize(wid, hei) => {
                    self.width = wid as u32;
                    self.height = hei as u32;
                    unsafe {
                        gl::Viewport(0, 0, wid, hei);
                    }
                }
                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    self.game.as_mut().unwrap().cursor_pos(xpos, ypos);
                }
                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
                    if key == Key::Escape {
                        self.window.write().unwrap().set_cursor_mode(glfw::CursorMode::Normal);
                        self.game.as_mut().unwrap().set_mouse_focused(false);
                    }
                    self.game.as_mut().unwrap().keyboard(key, action);
                }
                glfw::WindowEvent::Scroll(_x, y) => {
                    self.game.as_mut().unwrap().scroll(y);
                }
                _ => {}
            }
        }

        self.window.write().unwrap().swap_buffers();
    }
}
