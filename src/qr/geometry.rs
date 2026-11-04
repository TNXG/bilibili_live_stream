//! Square-preserving geometry for QR rendering.
//!
//! Every geometry maps the QR module grid (including the 2-module quiet
//! border) onto terminal cells so that one module is physically square:
//! the black/white block renderer keeps the module aspect ratio exact in
//! cells, and the graphics raster picks an integer pixel size per module
//! whose canvas is a whole multiple of the terminal cell size.

use ratatui::layout::Size;

fn checked(side: u32, cell: Size, area: Size) -> Option<(u32, u32)> {
    if side == 0 || side > 512 {
        return None;
    }
    if cell.width == 0 || cell.height == 0 || area.width == 0 || area.height == 0 {
        return None;
    }
    Some((u32::from(cell.width), u32::from(cell.height)))
}

/// Geometry for the unicode half-block renderer: every module covers
/// `module_columns` cells wide and `module_halfrows` half-rows tall, so a
/// module is always a physical square on screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockGeometry {
    cells: Size,
    module_columns: u32,
    module_halfrows: u32,
}

impl BlockGeometry {
    /// Largest integer module scale that fits `area` cells at `cell` pixel size.
    pub fn fit(side: u32, cell: Size, area: Size) -> Option<Self> {
        let (cell_w, cell_h) = checked(side, cell, area)?;
        // Whole half-rows must pair into cells on every row.
        if side % 2 != 0 {
            return None;
        }
        // Module square in pixels: columns * cell_w == halfrows * cell_h / 2.
        let divisor = gcd(2 * cell_w, cell_h);
        let columns = cell_h / divisor;
        let halfrows = 2 * cell_w / divisor;
        let scale = u32::from(area.width)
            .checked_div(side.checked_mul(columns)?)?
            .min((2 * u32::from(area.height)).checked_div(side.checked_mul(halfrows)?)?);
        if scale == 0 {
            return None;
        }
        let module_columns = columns.checked_mul(scale)?;
        let module_halfrows = halfrows.checked_mul(scale)?;
        if module_columns.checked_mul(cell_w)?.checked_mul(2)?
            != module_halfrows.checked_mul(cell_h)?
        {
            return None;
        }
        let width = u16::try_from(side.checked_mul(module_columns)?).ok()?;
        let height = u16::try_from(side.checked_mul(module_halfrows)? / 2).ok()?;
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            cells: Size::new(width, height),
            module_columns,
            module_halfrows,
        })
    }

    pub fn cells(&self) -> Size {
        self.cells
    }

    /// Smallest lossless block layout: one cell per module column and one
    /// half-row per module half-row, saturated to the terminal size limits.
    pub fn minimum(side: u32, cell: Size) -> Option<Size> {
        if side == 0 || side % 2 != 0 || cell.width == 0 || cell.height == 0 {
            return None;
        }
        let divisor = gcd(2 * u32::from(cell.width), u32::from(cell.height));
        let columns = u32::from(cell.height) / divisor;
        let halfrows = 2 * u32::from(cell.width) / divisor;
        let width = u16::try_from(side.checked_mul(columns)?).unwrap_or(u16::MAX);
        let height = u16::try_from(side.checked_mul(halfrows)? / 2).unwrap_or(u16::MAX);
        Some(Size::new(width.max(1), height.max(1)))
    }

    pub fn module_columns(&self) -> u32 {
        self.module_columns
    }

    pub fn module_halfrows(&self) -> u32 {
        self.module_halfrows
    }
}

/// Geometry for the graphics (Sixel/Kitty/iTerm2) raster: each module is
/// `module_pixels` square pixels, the canvas is a physical square, and the
/// canvas is a whole multiple of the terminal cell size so no cell is partial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphicsGeometry {
    cells: Size,
    pixels: u32,
    module_pixels: u32,
    offset: u32,
}

impl GraphicsGeometry {
    /// Largest integer module scale whose square canvas fits `area` cells.
    pub fn fit(side: u32, cell: Size, area: Size) -> Option<Self> {
        let (cell_w, cell_h) = checked(side, cell, area)?;
        let lcm = lcm(cell_w, cell_h);
        // Bounded so a huge terminal cannot allocate an unbounded raster.
        let max_pixels = u32::from(area.width)
            .checked_mul(cell_w)?
            .min(u32::from(area.height).checked_mul(cell_h)?)
            .min(2048);
        if lcm == 0 || lcm > max_pixels {
            return None;
        }
        // Largest cell-aligned square canvas whose modules are whole pixels.
        for steps in (1..=max_pixels / lcm).rev() {
            let canvas = steps * lcm;
            let module_pixels = canvas / side;
            if module_pixels == 0 || (canvas - module_pixels * side) % 2 != 0 {
                continue;
            }
            let offset = (canvas - module_pixels * side) / 2;
            let width = u16::try_from(canvas / cell_w).ok()?;
            let height = u16::try_from(canvas / cell_h).ok()?;
            if width == 0 || height == 0 {
                return None;
            }
            return Some(Self {
                cells: Size::new(width, height),
                pixels: canvas,
                module_pixels,
                offset,
            });
        }
        None
    }

    /// Smallest lossless graphics layout: the smallest cell-aligned square
    /// canvas that still holds every module at one whole pixel.
    pub fn minimum(side: u32, cell: Size) -> Option<Size> {
        if side == 0 || cell.width == 0 || cell.height == 0 {
            return None;
        }
        let lcm = lcm(u32::from(cell.width), u32::from(cell.height));
        if lcm == 0 {
            return None;
        }
        for factor in 1..=8_u32 {
            let canvas = lcm.checked_mul(factor)?;
            let module = canvas / side;
            if module == 0 {
                continue;
            }
            if (canvas - module * side) % 2 == 0 {
                let width = u16::try_from(canvas / u32::from(cell.width)).ok()?;
                let height = u16::try_from(canvas / u32::from(cell.height)).ok()?;
                return Some(Size::new(width.max(1), height.max(1)));
            }
        }
        None
    }

    pub fn cells(&self) -> Size {
        self.cells
    }

    /// Square canvas edge in pixels.
    pub fn pixels(&self) -> u32 {
        self.pixels
    }

    pub fn module_pixels(&self) -> u32 {
        self.module_pixels
    }

    /// Quiet-zone offset in pixels (two modules).
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: u32, b: u32) -> u32 {
    a.checked_mul(b / gcd(a, b)).unwrap_or(0)
}
