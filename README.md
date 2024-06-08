# term-gfx-rs

Terminal graphics engine written in rust, written for exchangability of `Canvas` and `App` types to allow for benchmarking.

## aims

- use as few libraries as necessary:
  - will likely keep [nalgebra](https://github.com/dimforge/nalgebra)
  - move away from input libraries as a whole, handle it ourselves
  - move away from [crossterm](https://github.com/crossterm-rs/crossterm) if possible
- write benchmarkable code
  - keeping types as traits when possible allows us to
    swap and replace components as we need for more
    comprehensive testing.
