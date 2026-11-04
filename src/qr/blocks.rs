//! Black/white half-block QR painter: exact module squares, no antialiasing.
//!
//! One module is `module_columns` cells wide and `module_halfrows` half-rows
//! tall, which the geometry guarantees to be physically square for the
//! terminal's cell size. Every cell stacks two module half-rows.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

use super::{BlockGeometry, ModuleGrid};

pub(super) fn render(buffer: &mut Buffer, area: Rect, grid: &ModuleGrid, geometry: BlockGeometry) {
    let cells = geometry.cells();
    for row in 0..u32::from(cells.height) {
        for column in 0..u32::from(cells.width) {
            let top = module_at(grid, geometry, column, row * 2);
            let bottom = module_at(grid, geometry, column, row * 2 + 1);
            let symbol = match (top, bottom) {
                (true, true) => "█",
                (true, false) => "▀",
                (false, true) => "▄",
                (false, false) => " ",
            };
            let style = Style::default().fg(Color::Black).bg(Color::White);
            buffer[(
                area.x + u16::try_from(column).unwrap_or(u16::MAX),
                area.y + u16::try_from(row).unwrap_or(u16::MAX),
            )]
                .set_symbol(symbol)
                .set_style(style);
        }
    }
}

/// Sample the module grid at the center of a cell's module half-row.
fn module_at(grid: &ModuleGrid, geometry: BlockGeometry, column: u32, halfrow: u32) -> bool {
    let cells = geometry.cells();
    let side = grid.block_side();
    let x = ((2 * column + 1) * side) / (2 * cells.width.max(1) as u32);
    let y = ((2 * halfrow + 1) * side) / (2 * cells.height.max(1) as u32 * 2);
    grid.dark(x, y)
}
