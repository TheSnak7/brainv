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

Measured on 2025-05-10.

Times include compilation, runtime setup, execution and cleanup. Cranelift uses its default `opt_level = none`.

### Interpreter and custom JIT

| Version     | Primes(350) | Pi-Digits(150) | Mandelbrot tiny | Sierpinski |
|-------------|-------------|----------------|-----------------|------------|
| Interpreter | 360.67 ms   | 119.81 ms      | 2.629 s         | 93.06 µs   |
| Custom JIT  | 43.28 ms    | 12.63 ms       | 287.55 ms       | 16.72 µs   |

### Custom JIT and Cranelift

| Version       | Primes(350) | Pi-Digits(150) | Mandelbrot tiny | Sierpinski |
|---------------|-------------|----------------|-----------------|------------|
| Custom JIT    | 45.32 ms    | 13.11 ms       | 270.20 ms       | 19.93 µs   |
| Cranelift JIT | 48.29 ms    | 18.28 ms       | 267.03 ms       | 1.313 ms   |
