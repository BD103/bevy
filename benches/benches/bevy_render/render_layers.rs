use core::hint::black_box;

use benches::bench;
use bevy_render::view::RenderLayers;
use criterion::{criterion_group, Criterion};

fn intersects(c: &mut Criterion) {
    c.bench_function(bench!("intersects"), |b| {
        let layer_a = black_box(RenderLayers::layer(1).with(2));
        let layer_b = black_box(RenderLayers::layer(1));

        b.iter(|| {
            let intersects = layer_a.intersects(&layer_b);

            #[cfg(test)]
            assert!(intersects);

            intersects
        });
    });
}

criterion_group!(benches, intersects);
