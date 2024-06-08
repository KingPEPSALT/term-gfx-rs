#![allow(incomplete_features)] // reason: generic_const_exprs
#![feature(
    generic_const_exprs,
    iter_array_chunks,
    ascii_char,
    trait_alias,
    duration_millis_float
)]

use std::io;

use app::{App, Application};
mod app;
mod graphics;

fn main() -> Result<(), io::Error> {
    let thread = std::thread::Builder::new()
        .name("main".to_string())
        .stack_size(32 * 1024 * 1024); // 32MiB stack-size

    let handler = thread.spawn(run::<App>)?;

    handler.join().unwrap()
}

fn run<'a, A: Application<'a>>() -> Result<(), A::Error> {
    let mut app = A::fresh("rstracer");

    app.initialise()?;
    while app.is_running() {
        app.clear()?;

        app.begin_frame()?;

        app.input()?;
        app.update()?;
        app.render()?;

        app.end_frame()?;
    }
    app.end()?;
    Ok(())
}
