use fixedbitset::FixedBitSet;
use js_sys::Math;
use std::fmt;
use wasm_bindgen::prelude::*;
use web_sys;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

impl Universe {
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.width - 1, 0, 1].iter().cloned() {
            for delta_col in [self.height - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0 {
                    continue;
                }

                let idx = self.get_index(
                    (row + delta_row) % self.width,
                    (col + delta_col) % self.height,
                );
                count += self.cells[idx] as u8
            }
        }
        count
    }
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.width {
            for col in 0..self.height {
                let live_cnt = self.live_neighbor_count(row, col);
                let idx = self.get_index(row, col);
                let current_state = self.cells[idx];

                let next_state = match (current_state, live_cnt) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (true, n) if n < 2 => false,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (true, n) if n > 3 => false,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (some_state, _) => some_state,
                };
                next.set(idx, next_state);

                if current_state != next_state {
                    log!(
                        "cell ({:?}, {:?}) transitioned to {:?} state",
                        row,
                        col,
                        if next_state { "live" } else { "dead" }
                    );
                }
            }
        }

        self.cells = next
    }

    pub fn new(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();
        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        for idx in 0..size {
            cells.set(idx, Math::random() >= 0.5);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn new_copperhead(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();
        let top_offset = 32;
        let left_offset = 32;
        let copperhead = [
            false, true, true, false, false, true, true, false, false, false, false, true, true,
            false, false, false, false, false, false, true, true, false, false, false, true, false,
            true, false, false, true, false, true, true, false, false, false, false, false, false,
            true, false, false, false, false, false, false, false, false, true, false, false,
            false, false, false, false, true, false, true, true, false, false, true, true, false,
            false, false, true, true, true, true, false, false, false, false, false, false, false,
            false, false, false, false, false, false, true, true, false, false, false, false,
            false, false, true, true, false, false, false,
        ];

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);
        let mut ship_row = 0;
        for cells_row in top_offset + 1..=top_offset + copperhead.len() / 8 {
            let row_offset = ship_row * 8;
            for ship_col in 0..8 {
                cells.set(
                    cells_row * width as usize + left_offset + ship_col + 1,
                    copperhead[row_offset + ship_col],
                );
            }
            ship_row += 1;
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Set the width of the universe.
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = FixedBitSet::with_capacity((width * self.height) as usize);
    }

    /// Set the height of the universe.
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = FixedBitSet::with_capacity((self.width * height) as usize);
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell != 0 { '◼' } else { '◻' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

impl Universe {
    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
}
