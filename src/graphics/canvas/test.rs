#![cfg(test)]
#![allow(dead_code)]
use super::*;
use std::io::BufWriter;

fn test_canvas() -> Result<(), std::io::Error> {
    let mut canvas = BufferedCanvas::<20, 20, 3>::new();
    canvas.put_pixel(Colour::new(255, 255, 100), Vector2::new(10, 10));
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = BufWriter::new(lock);
    canvas.full_swap();
    canvas.display(&mut writer)
}

fn test_eols() {
    use super::BufferedCanvas;

    let mut canvas = BufferedCanvas::<20, 20, 3>::new();
    canvas.put_pixel(Colour::new(255, 255, 100), Vector2::new(10, 10));

    for buffer in 0..canvas.buffers() {
        for row in 0..canvas.size().y {
            assert_eq!(
                canvas._buffers[buffer]
                    [(row + 1) * BufferedCanvas::<20, 20, 3>::internal_width() - 1],
                b'\n'
            );
        }
    }
}
