use core::hint::black_box;

use benches::bench;
use criterion::{criterion_group, Criterion};
use bevy_render::mesh::{MeshBuilder, TorusMeshBuilder};

fn torus(c: &mut Criterion) {
    c.bench_function(bench!("torus"), |b| {
        let builder = black_box(TorusMeshBuilder::new(0.5, 1.0));
        b.iter(|| builder.build());
    });
}

criterion_group!(benches, torus);
