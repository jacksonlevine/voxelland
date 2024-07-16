


use criterion::{black_box, criterion_group, criterion_main, Criterion};
use voxelland::{chunk::*};


fn criterion_benchmark(c: &mut Criterion) {

    let width = 1280;
    let height = 720;
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let (mut window, _events) = glfw
        .create_window(width, height, "Hello", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");
    gl::load_with(|s| window.get_proc_address(s) as *const _);


    let csys = ChunkSystem::new(8, 1, 0, false, None);

    c.bench_function("rebuild 20 chunks", |b| b.iter(|| csys.rebuild_index(black_box(20), false, false)));
}



criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);