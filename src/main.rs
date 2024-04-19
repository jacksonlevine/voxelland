use glfw::{Action, Context, Key};
use windowandkey::WindowAndKeyContext;
use std::time::{Instant};
use game::Game;

mod vec;
mod shader;
mod worldgeometry;
mod chunk;
mod cube;
mod packedvertex;
mod windowandkey;
mod game;

fn main() {
    let mut wak_context = WindowAndKeyContext::new("Barkaroo");
    let mut game = Game::new();

    wak_context.game = Some(game);

    while !wak_context.window.should_close() {
        wak_context.run();
    }
}