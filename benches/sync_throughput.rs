//! Benchmark harness template for sync throughput.
//!
//! Move this into a benchmark crate when simulator adapters are available.

use criterion::{criterion_group, criterion_main, Criterion};

fn sync_delta_throughput(c: &mut Criterion) {
    c.bench_function("sync_delta_1000_objects", |b| {
        b.iter(|| {
            // Build two in-memory devices.
            // Insert 1000 changed objects.
            // Run sync over encrypted loopback transport.
            // Assert convergence outside the timed inner loop in the real bench.
        });
    });
}

criterion_group!(benches, sync_delta_throughput);
criterion_main!(benches);
