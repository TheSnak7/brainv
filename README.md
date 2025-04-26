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

| Version           | Primes(350) | Pi-Digits(150) | Mandelbrot(100) |
|-------------------|-------------|----------------|-----------------|
| Simple            | 747.49 ms   | 304.30  ms     | -               |
| Condensed         | 356.18 ms   | 126.99  ms     | -               |
| Precomputed Jumps | 303.92 ms   |  97.933 ms     | 2.5503 s        |
| Cranelift JIT     | 304.55 ms   |  98.117 ms     | 2.4558 s        |
| Custom JIT        | 299.99 ms   |  97.556 ms     | 2.3874 s        |
