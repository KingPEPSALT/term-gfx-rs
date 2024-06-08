mod buffered_canvas;
mod test;
use std::io::Write;


use nalgebra::Vector2;

use crate::graphics::Colour;

#[allow(unused_imports)]
pub use self::buffered_canvas::BufferedCanvas;

/// Converts a byte, i.e. `255` to its digits in base ten: [b'2', b'5', b'5']
/// or `10` to [b'0', b'1', b'0']
pub(self) fn btod(num: u8) -> [u8; 3] {
    let hundreds = num / 100;
    let tens = (num - hundreds * 100) / 10;
    let units = num - hundreds * 100 - tens * 10;
    [48 + hundreds, 48 + tens, 48 + units]
}

pub trait Canvas {
    type Writer: Write;
    /// Puts a pixel to the canvas, is an empty cell in reality
    fn put_pixel(&mut self, colour: Colour, position: Vector2<usize>);
    /// Fills the canvas with empty cells of given colour
    fn fill(&mut self, colour: Colour);
    /// Fills the canvas with black
    fn clear(&mut self);
    /// Returns the size of the canvas
    fn size(&self) -> Vector2<usize>;
    /// Displays the canvas to `out`
    fn display(&self, out: &mut dyn Write) -> Result<(), std::io::Error>;
    /// Write a string to the canvas, is split into cells
    fn write(&mut self, str: String, colour: Colour, canvas_index: Vector2<usize>);
    /// Put a single cell of two characters to the canvas
    fn put_cell(&mut self, cell: [u8; 2], colour: Colour, canvas_index: Vector2<usize>);
}
