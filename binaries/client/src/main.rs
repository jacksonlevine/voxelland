use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::Subscriber;
use std::fs::File;

use voxelland::windowandkey::WindowAndKeyContext;

use voxelland::game::Game;



fn main() {

    let file = File::create("app.log").unwrap();
    let make_writer = BoxMakeWriter::new(file);

    let subscriber = Subscriber::builder()
        .with_writer(make_writer)
        .with_span_events(FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let mut wak_context = WindowAndKeyContext::new("Barkaroo", 1280, 720);

    let mut gameh = Game::new(&wak_context.window, true, false);

    while !gameh.is_finished() {
        wak_context.run();
    }

    let mut game: Game;

    match gameh.join() {
        Ok(gamei) => {
            game = gamei;
        }
        Err(e) => {
            panic!("Jumbotron Shit Broken");
        }
    }

    wak_context.game = Some(game);

    let handle = wak_context.game.as_mut().unwrap().initialize_being_in_world();

    while !handle.is_finished() {
        wak_context.run();
    }

    match handle.join() {
        Ok(_) => {
            wak_context.game.as_mut().unwrap().loadedworld.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Err(e) => {

        }
    }
    wak_context.game.as_mut().unwrap().vars.menu_open = false;
    
    wak_context.game.as_mut().unwrap().start_world();
    while !wak_context.window.read().unwrap().should_close() {
        wak_context.run();
    }
}
