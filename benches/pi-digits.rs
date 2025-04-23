use criterion::{criterion_group, criterion_main, Criterion};
use brainv::vm::bench_run;

pub fn criterion_benchmark(c: &mut Criterion) {
    let code = include_str!("bf/pi-digits.bf");
    c.bench_function("pi-digits", |b| b.iter(|| bench_run(code, "90\n".as_bytes().into())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);