use std::hint::black_box;

use bevy_ecs::world::World;
use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig, Tool, ValgrindTool
};

#[library_benchmark]
fn bench_world_new() {
    let world = World::new();
    black_box(world);
}

library_benchmark_group!(
    name = ecs_group;
    benchmarks = bench_world_new
);

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Tool::new(ValgrindTool::DHAT))
        .tool(Tool::new(ValgrindTool::Massif))
        .tool(Tool::new(ValgrindTool::Memcheck))
        .flamegraph(FlamegraphConfig::default());
    library_benchmark_groups = ecs_group
);
