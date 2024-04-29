
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use voxelland::{camera::Camera, chunk::*, game::Game};


fn criterion_benchmark(c: &mut Criterion) {

    let width = 1280;
    let height = 720;
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, events) = glfw
        .create_window(width, height, "Hello", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let csys = ChunkSystem::new(8);

    c.bench_function("rebuild 20 chunks", |b| b.iter(|| csys.rebuild_index(black_box(20), false)));
}

fn criterion_benchmark2(c: &mut Criterion) {
    let width = 1280;
    let height = 720;
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, events) = glfw
        .create_window(width, height, "Hello", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    let csys = ChunkSystem::new(8);

    let cam_arc: Arc<Mutex<Camera>> = Arc::new(Mutex::new(Camera::new()));
    let csys_arc: Arc<ChunkSystem> = Arc::new(csys);

    let mut group = c.benchmark_group("Chunk System Performance");

    group.sample_size(10);

    group.bench_function("run chunk thread once", |b| {
        b.iter(|| Game::chunk_thread_inner_function(&cam_arc, &csys_arc))
    });

    group.finish();
}


criterion_group!(benches, criterion_benchmark, criterion_benchmark2);
criterion_main!(benches);