use criterion::criterion_main;

mod render_layers;
mod mesh_builder;

criterion_main!(render_layers::benches, mesh_builder::benches);
