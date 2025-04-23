use brainv::bench_run;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let code = include_str!("bf/serptri.bf");
    c.bench_function("serptri", |b| b.iter(|| bench_run(code, vec![])));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);