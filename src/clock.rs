pub const CLOCK_CHAR: char = '╱';

pub const DIGIT_WIDTH: usize = 6;
pub const DIGIT_HEIGHT: usize = 7;
pub const GAP_DIGIT: usize = 2;
pub const GAP_COLON: usize = 2;

// 6x7 bitmap for digits 0-9
// 1 = CLOCK_CHAR, 0 = space
const FONT: [[[u8; DIGIT_WIDTH]; DIGIT_HEIGHT]; 10] = [
    // 0
    [
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
    // 1
    [
        [1, 1, 1, 1, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 0, 0],
        [1, 1, 1, 1, 1, 1],
    ],
    // 2
    [
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1],
    ],
    // 3
    [
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
    // 4
    [
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
    ],
    // 5
    [
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
    // 6
    [
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
    // 7
    [
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
    ],
    // 8
    [
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
    // 9
    [
        [1, 1, 1, 1, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [0, 0, 0, 0, 1, 1],
        [1, 1, 1, 1, 1, 1],
    ],
];

// 2x7 bitmap for colon
const COLON: [[u8; 2]; DIGIT_HEIGHT] = [
    [0, 0],
    [1, 1],
    [1, 1],
    [0, 0],
    [1, 1],
    [1, 1],
    [0, 0],
];

/// Step 1: Render unslanted grid of cells (7 rows high)
pub fn render_unslanted_clock(hours: u32, minutes: u32) -> Vec<String> {
    let d1 = ((hours / 10) % 10) as usize;
    let d2 = (hours % 10) as usize;
    let d3 = ((minutes / 10) % 10) as usize;
    let d4 = (minutes % 10) as usize;

    let mut rows = Vec::with_capacity(DIGIT_HEIGHT);

    for r in 0..DIGIT_HEIGHT {
        let mut line = String::new();

        // Digit 1 (H1)
        for c in 0..DIGIT_WIDTH {
            line.push(if FONT[d1][r][c] == 1 { CLOCK_CHAR } else { ' ' });
        }
        // Gap
        line.push_str(&" ".repeat(GAP_DIGIT));

        // Digit 2 (H2)
        for c in 0..DIGIT_WIDTH {
            line.push(if FONT[d2][r][c] == 1 { CLOCK_CHAR } else { ' ' });
        }
        // Gap to colon (symmetrical)
        line.push_str(&" ".repeat(GAP_COLON));

        // Colon (:)
        for c in 0..2 {
            line.push(if COLON[r][c] == 1 { CLOCK_CHAR } else { ' ' });
        }
        // Gap from colon (symmetrical)
        line.push_str(&" ".repeat(GAP_COLON));

        // Digit 3 (M1)
        for c in 0..DIGIT_WIDTH {
            line.push(if FONT[d3][r][c] == 1 { CLOCK_CHAR } else { ' ' });
        }
        // Gap
        line.push_str(&" ".repeat(GAP_DIGIT));

        // Digit 4 (M2)
        for c in 0..DIGIT_WIDTH {
            line.push(if FONT[d4][r][c] == 1 { CLOCK_CHAR } else { ' ' });
        }

        rows.push(line);
    }

    rows
}

/// Step 2: Slant the grid by shifting rows:
/// Row 0 is shifted +3, Row 3 is 0, Row 6 is -3.
/// Pads each row to uniform total width to ensure perfect centering.
pub fn slant_grid(grid: &[String]) -> Vec<String> {
    let max_row = grid.len().saturating_sub(1);
    let mut slanted = Vec::with_capacity(grid.len());

    for (r, line) in grid.iter().enumerate() {
        let pad_left = max_row - r; // 6, 5, 4, 3, 2, 1, 0
        let pad_right = r;          // 0, 1, 2, 3, 4, 5, 6

        let mut row_str = String::new();
        row_str.push_str(&" ".repeat(pad_left));
        row_str.push_str(line);
        row_str.push_str(&" ".repeat(pad_right));

        slanted.push(row_str);
    }

    slanted
}

/// Helper: Render complete slanted clock
pub fn render_slanted_clock(hours: u32, minutes: u32) -> Vec<String> {
    let unslanted = render_unslanted_clock(hours, minutes);
    slant_grid(&unslanted)
}
