<p align="center">
  <a href="https://github.com/TheSnak7/brainv"><img src="logo.png" alt="Brainv Logo" height=170></a>
</p>
<h1 align="center">Brainv</h1>

<p align="center">
  <a href="https://github.com/TheSnak7/brainv"><img src="https://img.shields.io/github/stars/TheSnak7/brainv" alt="stars"></a>
</p>

## About

Brainv is a brainf*ck interpreter for me to explore some libraries and maybe some optimization strategies. The different stages are available as different branches.

## Benchmarks

Measured on 2025-05-10.

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
