use glfw::Context;
use windowandkey::WindowAndKeyContext;

use game::Game;

mod blockinfo;
mod camera;
mod chunk;
mod collisioncage;
mod cube;
mod fader;
mod game;
mod packedvertex;
mod shader;
mod texture;
mod vec;
mod windowandkey;
mod worldgeometry;

#[cfg(test)]
mod tests;

fn main() {
    let mut wak_context = WindowAndKeyContext::new("Barkaroo");

    let game = Game::new();

    wak_context.game = Some(game);
    wak_context.game.as_mut().unwrap().set_mouse_focused(true);
    wak_context.game.as_mut().unwrap().start_world();
    while !wak_context.window.should_close() {
        wak_context.run();
    }
}
