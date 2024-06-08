# term-gfx-rs

Terminal graphics engine written in rust, written for exchangability of `Canvas` and `App` types to allow for benchmarking.

## disclaimer

This project is massively underfinished and basically only just runs, the code is currently extremely messy. I have
only taken the liberty of removing the obvious warnings but until more implementation, I don't see a reason to clean
the code.

## running

It's best to run in release. zoom your terminal out, really far out, or change `BufferedCanvas<W, H>` in `App`.
`BufferedCanvas` also doesn't really buffer right now and will likely be scrapped.

```bash
cargo run --release
```

## aims

- use as few libraries as necessary:
  - will likely keep [nalgebra](https://github.com/dimforge/nalgebra)
  - move away from input libraries as a whole, handle it ourselves
  - move away from [crossterm](https://github.com/crossterm-rs/crossterm) if possible
- write benchmarkable code
  - keeping types as traits when possible allows us to
    swap and replace components as we need for more
    comprehensive testing.
