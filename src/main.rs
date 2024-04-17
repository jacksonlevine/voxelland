use glfw::{Action, Context, Key};
use std::time::{Instant};

mod vec;
mod shader;
mod worldgeometry;
mod chunk;
mod cube;
mod packedvertex;

fn main() {
    let mut width = 1280;
    let mut height = 720;

    let mut previous_time = Instant::now();
    let mut delta_time: f32 = 0.0;


    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, events) = glfw.create_window(width, height, "Project Codename WaxJab RS FoxTrot", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_mouse_button_polling(true);

    while !window.should_close() {
        glfw.poll_events();

        let current_time = Instant::now();
        delta_time = current_time.duration_since(previous_time).as_secs_f32();
        previous_time = current_time;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }


        for (_, event) in glfw::flush_messages(&events) {
            match event {

                glfw::WindowEvent::MouseButton(_mousebutton, _action, _) => {
                    
                },
                glfw::WindowEvent::FramebufferSize(wid, hei) => {
                    width = wid as u32;
                    height = hei as u32;
                    unsafe {
                        gl::Viewport(0, 0, wid, hei);
                    }
                }
                _ => {}
            }
        }
        

        window.swap_buffers();
    }
}
