use core::hint::black_box;

use benches::bench;
use bevy_math::{
    prelude::{CircularSector, CircularSegment},
    Dir3, Vec2,
};
use bevy_render::mesh::{
    AnnulusMeshBuilder, Capsule2dMeshBuilder, Capsule3dMeshBuilder, CircleMeshBuilder,
    CircularSectorMeshBuilder, CircularSegmentMeshBuilder, ConeMeshBuilder,
    ConicalFrustumMeshBuilder, CylinderMeshBuilder, EllipseMeshBuilder, MeshBuilder,
    PlaneMeshBuilder, SphereKind, SphereMeshBuilder, TorusMeshBuilder,
};
use criterion::{criterion_group, measurement::Measurement, Bencher, Criterion};

/// A helper function that benchmarks running [`MeshBuilder::build()`] on a specific mesh builder.
fn bench_mesh_builder<M: Measurement, B: MeshBuilder>(b: &mut Bencher<M>, builder: B) {
    let builder = black_box(builder);
    b.iter(|| builder.build());
}

fn mesh_builder(c: &mut Criterion) {
    c.bench_function(bench!("annulus"), |b| {
        bench_mesh_builder(b, AnnulusMeshBuilder::new(0.5, 1.0, 32));
    });

    c.bench_function(bench!("capsule_2d"), |b| {
        bench_mesh_builder(b, Capsule2dMeshBuilder::new(0.5, 1.0, 16));
    });

    c.bench_function(bench!("capsule_3d"), |b| {
        bench_mesh_builder(b, Capsule3dMeshBuilder::new(0.5, 0.5, 32, 16));
    });

    c.bench_function(bench!("circle"), |b| {
        bench_mesh_builder(b, CircleMeshBuilder::new(1.0, 32));
    });

    c.bench_function(bench!("circular_sector"), |b| {
        bench_mesh_builder(
            b,
            CircularSectorMeshBuilder::new(CircularSector::new(1.0, 1.0)),
        );
    });

    c.bench_function(bench!("circular_segment"), |b| {
        bench_mesh_builder(
            b,
            CircularSegmentMeshBuilder::new(CircularSegment::new(1.0, 1.0)),
        );
    });

    c.bench_function(bench!("cone"), |b| {
        bench_mesh_builder(b, ConeMeshBuilder::new(0.5, 1.0, 32));
    });

    c.bench_function(bench!("conical_frustum"), |b| {
        bench_mesh_builder(b, ConicalFrustumMeshBuilder::new(0.5, 1.0, 2.0, 32));
    });

    c.bench_function(bench!("cylinder"), |b| {
        bench_mesh_builder(b, CylinderMeshBuilder::new(0.5, 1.0, 32));
    });

    c.bench_function(bench!("ellipse"), |b| {
        bench_mesh_builder(b, EllipseMeshBuilder::new(0.5, 1.0, 32));
    });

    c.bench_function(bench!("plane"), |b| {
        bench_mesh_builder(b, PlaneMeshBuilder::new(Dir3::X, Vec2::ONE));
    });

    {
        let mut group = c.benchmark_group(bench!("sphere"));

        group.bench_function("ico", |b| {
            bench_mesh_builder(
                b,
                SphereMeshBuilder::new(1.0, SphereKind::Ico { subdivisions: 5 }),
            );
        });

        group.bench_function("uv", |b| {
            bench_mesh_builder(
                b,
                SphereMeshBuilder::new(
                    1.0,
                    SphereKind::Uv {
                        sectors: 32,
                        stacks: 18,
                    },
                ),
            );
        });
    }

    c.bench_function(bench!("torus"), |b| {
        bench_mesh_builder(b, TorusMeshBuilder::new(0.5, 1.0));
    });

    // TODO: Rectangles, regular polygons, rhombuses, triangle 2d, and convex polygons cannot be
    // constructed.
}

criterion_group!(benches, mesh_builder);
