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

    let mut wak_context = WindowAndKeyContext::new("Barkaroo");

    let game = Game::new();

    wak_context.game = Some(game);
    wak_context.game.as_mut().unwrap().start_world();
    while !wak_context.window.should_close() {
        wak_context.run();
    }
}
