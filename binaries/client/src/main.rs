
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::Subscriber;
use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing::info;
use std::fs::File;

use voxelland::windowandkey::WindowAndKeyContext;

use voxelland::game::{Game, DECIDEDSPORMP};



fn main() {

    // Create a non-blocking, asynchronous file writer
    let file = File::create("app.log").expect("Unable to create log file");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    // Create a tracing subscriber with the non-blocking writer
    let subscriber = Subscriber::builder()
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");


    let mut wak_context = WindowAndKeyContext::new("Distant Garden", 1280, 720);

    unsafe {
        while !DECIDEDSPORMP {
            if !wak_context.window.read().unwrap().should_close() {
                wak_context.run();
            } else {
                return ();
            }
        }
    }
    

    

    let gameh = Game::new(&wak_context.window, true, false, &wak_context.addressentered, &wak_context.serveraddress);

    while !gameh.is_finished() {
        if !wak_context.window.read().unwrap().should_close() {
            wak_context.run();
        } else {
            return ();
        }
        
    }


    let game: Game;

    match gameh.join() {
        Ok(gamei) => {
            game = gamei;
        }
        Err(_e) => {
            panic!("Failed to create Game.");
        }
    }

    info!("gltf model count: {}", game.gltf_models.len());

    wak_context.game = Some(game);

    let handle = wak_context.game.as_mut().unwrap().initialize_being_in_world();

    while !handle.is_finished() {
        if !wak_context.window.read().unwrap().should_close() {
            wak_context.run();
        } else {
            return ();
        }
    }

    match handle.join() {
        Ok(_) => {
            wak_context.game.as_mut().unwrap().loadedworld.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Err(_e) => {

        }
    }
    wak_context.game.as_mut().unwrap().vars.menu_open = false;
    
    wak_context.game.as_mut().unwrap().start_world();
    while !wak_context.window.read().unwrap().should_close() {
        wak_context.run();
    }
}
