use binary_search_bench::{BenchInput, Variant, run_variant};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

fn bench_variant(c: &mut Criterion) {
    const MIN_EXP: u32 = 10;
    const MAX_EXP: u32 = 22;
    const QUERIES: usize = 16_384;
    const SEED: u64 = 42;

    let mut group = c.benchmark_group("binary_search_variants");
    group.sample_size(20);

    for exp in MIN_EXP..=MAX_EXP {
        let size = 1usize << exp;
        let input = BenchInput::new(size, QUERIES, SEED ^ size as u64);
        group.throughput(Throughput::Elements(QUERIES as u64));

        for variant in Variant::ALL {
            group.bench_with_input(
                BenchmarkId::new(variant.name(), size),
                &input,
                |b, input| {
                    b.iter(|| black_box(run_variant(input, variant)));
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_variant);
criterion_main!(benches);
