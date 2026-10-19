<p align="center">
  <a href="https://github.com/TheSnak7/brainv"><img src="logo.png" alt="Brainv Logo" height=170></a>
</p>
<h1 align="center">Brainv</h1>

<p align="center">
  <a href="https://github.com/TheSnak7/brainv"><img src="https://img.shields.io/github/stars/TheSnak7/brainv" alt="stars"></a>
</p>

## About

Brainv is a brainf*ck interpreter for me to explore some libraries and maybe some optimization strategies. This project is written in Rust (maybe not the most idiomatic :) ) and uses different branches for the different modes.

It has an Interpreter which can naively interpret the brainf*ck, or precompute jumps. Also test coverage, which I did not skimp on this project.
The most interesting thing is the Arm64 JIT and emitted assembly, with memory management and ABI correctness tests.

## Requirements

- Windows ARM64 for the custom JIT on `main`.
- Rust 1.86.0 (tested).

## Usage

```powershell
cargo run --release -- bf_tests/hello_world.bf
```

Use `--io` to select the IO mode:

- `batched`: default, buffers output in 200-byte chunks.
- `simple`: prints each byte immediately.
- `one-print`: uses a larger output buffer for benchmarking.

## Branches

| Branch                   | Mode |
|--------------------------|------|
| `model/simple`           | Basic interpreter |
| `model/condensed`        | Combines repeated arithmetic and move instructions |
| `model/precompute-jumps` | Precomputes matching bracket jumps |
| `model/cranelift-jit`    | Cranelift JIT |
| `main`                   | Custom ARM64 JIT |

## Benchmarks

Times include compilation, runtime setup, execution and cleanup. Cranelift uses its default `opt_level = none`.

### 2026-10-19

Windows ARM64, Snapdragon X Elite X1E80100, Rust 1.86.0 in release mode. 10 samples per workload.

| Version       | Primes(350) | Pi-Digits(150) | Mandelbrot tiny | Sierpinski |
|---------------|-------------|----------------|-----------------|------------|
| Interpreter   | 695.68 ms   | 200.84 ms      | 3.391 s         | 149.97 µs  |
| Custom JIT    | 65.00 ms    | 20.58 ms       | 493.73 ms       | 30.10 µs   |
| Cranelift JIT | 63.56 ms    | 32.39 ms       | 520.02 ms       | 1.521 ms   |
