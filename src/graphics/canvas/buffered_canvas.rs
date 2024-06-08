use std::io::Write;

use nalgebra::Vector2;

use crate::{app::StdoutWriter, graphics::Colour};

use super::Canvas;

const PREFIX_SIZE: usize = 21;
const EOL_SIZE: usize = 5;
pub struct BufferedCanvas<'a, const WIDTH: usize, const HEIGHT: usize, const BUFFERS: usize>
where
    [(); WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT]:,
{
    pub(super) _buffers: [[u8; WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT]; BUFFERS],
    pub(super) _display_index: usize,
    pub(super) _edit_index: usize,
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const BUFFERS: usize>
    BufferedCanvas<'a, WIDTH, HEIGHT, BUFFERS>
where
    [(); WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT]:,
{
    /// Creates a buffered canvas prepared for rendering
    pub fn new() -> Self {
        let mut prepared_buffer = [0; WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT];
        for row in 1..HEIGHT + 1 {
            // write the reset and new line characters to create the nice box
            // and avoid overflowing style
            let idx = row * Self::internal_width() - EOL_SIZE;
            prepared_buffer[idx + 0] = b'\x1B';
            prepared_buffer[idx + 1] = b'[';
            prepared_buffer[idx + 2] = b'0';
            prepared_buffer[idx + 3] = b'm';
            prepared_buffer[idx + 4] = b'\n';
        }

        let mut this = Self {
            _buffers: [prepared_buffer; BUFFERS],
            _display_index: 0,
            _edit_index: BUFFERS - 1,
        };

        // prepare at least one buffer and swap to it
        this.clear();
        this.full_swap();
        this
    }

    /// Adjusts a buffer index, overflowing if it gets to the end,
    /// allows us to cycle through buffers
    fn adjust_index(index: &mut usize) {
        *index += 1;
        if *index == BUFFERS {
            *index = 0;
        }
    }

    #[inline]
    /// Returns the internal width of the buffer
    pub(super) fn internal_width() -> usize {
        Self::flatten_index(Vector2::y())
    }
    #[inline]
    /// Flattens the passed position to an index to before the cell styling
    pub(super) fn flatten_index(canvas_index: Vector2<usize>) -> usize {
        PREFIX_SIZE * canvas_index.x + canvas_index.y * (PREFIX_SIZE * WIDTH + EOL_SIZE) as usize
    }

    #[allow(dead_code)]
    #[inline]
    /// Returns the number of buffers
    pub fn buffers(&self) -> usize {
        BUFFERS
    }

    #[inline]
    /// Swap the buffers, fast
    pub fn swap(&mut self) {
        Self::adjust_index(&mut self._display_index);
        Self::adjust_index(&mut self._edit_index);
    }

    /// Swaps the currently edited buffer to the displayed buffer.
    /// Usually used before the first render.
    pub fn full_swap(&mut self) {
        let tmp = self._edit_index;
        self._edit_index = self._display_index;
        self._display_index = tmp;
    }
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const BUFFERS: usize> ToString
    for BufferedCanvas<'a, WIDTH, HEIGHT, BUFFERS>
where
    [(); WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT]:,
{
    fn to_string(&self) -> String {
        std::str::from_utf8(&self._buffers[self._display_index].map(|c| c as u8))
            .unwrap()
            .to_owned()
    }
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const BUFFERS: usize> Canvas
    for BufferedCanvas<'a, WIDTH, HEIGHT, BUFFERS>
where
    [(); WIDTH * HEIGHT * PREFIX_SIZE + EOL_SIZE * HEIGHT]:,
{
    type Writer = StdoutWriter<'a>;
    fn put_pixel(&mut self, colour: Colour, canvas_index: Vector2<usize>) {
        self.put_cell([b' '; 2], colour, canvas_index);
    }
    fn fill(&mut self, colour: Colour) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.put_pixel(colour, Vector2::new(x, y));
            }
        }
    }

    fn put_cell(&mut self, cell: [u8; 2], colour: Colour, canvas_index: Vector2<usize>) {
        // split colour into decimal digits
        let ns: [[u8; 3]; 3] = [colour.x, colour.y, colour.z].map(|b| super::btod(b));
        // cast buffer pointer into a pointer of different type so we can do std::mem::replace
        // in reality this is safe so long as the buffer size doesn't change and if it does,
        // this should crash and we'd know
        if canvas_index.y > HEIGHT || canvas_index.x > WIDTH {
            println!("{:#?}", canvas_index);
        }
        let ptr = unsafe {
            &mut *(&mut self._buffers[self._edit_index][Self::flatten_index(canvas_index)]
                as *mut u8 as *mut [u8; PREFIX_SIZE])
        };
        // generate the style prefix and the cell
        let _ = std::mem::replace(
            ptr,
            [
                b'\x1B', b'[', b'4', b'8', b';', b'2', b';', ns[0][0], ns[0][1], ns[0][2], b';',
                ns[1][0], ns[1][1], ns[1][2], b';', ns[2][0], ns[2][1], ns[2][2], b'm', cell[0],
                cell[1],
            ],
        );
    }

    fn write(&mut self, str: String, colour: Colour, canvas_index: Vector2<usize>) {
        // we just construct a new string with another square
        // of padding if not even length, this is because cells
        // are 2-chars wide.
        let mut adj_str = str;
        if adj_str.len() % 2 == 1 {
            adj_str += " ";
        }
        for (i, cell) in adj_str.bytes().array_chunks::<2>().enumerate() {
            self.put_cell(cell, colour, canvas_index + Vector2::new(i, 0));
        }
    }
    fn clear(&mut self) {
        // empty canvas, in a transparent canvas, this may
        // clear to no colour at all
        self.fill(Colour::from_element(0));
    }

    fn size(&self) -> Vector2<usize> {
        Vector2::new(WIDTH, HEIGHT)
    }

    fn display(&self, out: &mut dyn Write) -> Result<(), std::io::Error> {
        out.write_all(&self._buffers[self._display_index])
    }
}
