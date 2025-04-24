use criterion::{criterion_group, criterion_main, Criterion};
use brainv::vm::bench_run;

pub fn criterion_benchmark(c: &mut Criterion) {
    let code = include_str!("bf/primes.bf");

    let mut group = c.benchmark_group("small-programs");
    group.sample_size(200);
    group.bench_function("primes", |b| b.iter(|| bench_run(code, "350\n".as_bytes().into())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);