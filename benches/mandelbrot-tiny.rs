use criterion::{criterion_group, criterion_main, Criterion};
use brainv::vm::bench_run;

pub fn criterion_benchmark(c: &mut Criterion) {
    let code = include_str!("bf/mandelbrot-tiny.bf");
    c.bench_function("mandelbrot-tiny", |b| b.iter(|| bench_run(code, vec![])));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);