# Brainv

## About

Brainv is a brainf*ck interpreter for me to explore some libraries and maybe some optimization strategies.

## Benchmarks

| Version           | Primes(350) | Pi-Digits(150) | Mandelbrot(100) |
|-------------------|-------------|----------------|-----------------|
| Simple            | 747.49 ms   | 304.30  ms     | -               |
| Condensed         | 356.18 ms   | 126.99  ms     | -               |
| Precomputed Jumps | 303.92 ms   |  97.933 ms     | -               |
| Cranelift JIT     | 304.55 ms   |  98.117 ms     | 2.4558 s        |