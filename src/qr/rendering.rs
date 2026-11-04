//! Cached QR renderer: graphics protocol when the terminal supports it,
//! otherwise uniform black/white unicode blocks. Rendering never queries
//! stdio; the picker is captured once at startup.

use super::{
    blocks,
    geometry::{BlockGeometry, GraphicsGeometry},
    raster::ModuleGrid,
};

use ratatui::layout::Rect;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::{Image, Resize};

/// A QR payload prepared for the detected terminal; caches the encoded
/// protocol so repeated frames do not re-encode or re-scale.
pub struct QrDisplay {
    url: String,
    picker: Picker,
    mode: Mode,
}

enum Mode {
    Graphics {
        grid: ModuleGrid,
        protocol: Option<ratatui_image::protocol::Protocol>,
    },
    Blocks {
        grid: ModuleGrid,
    },
}

impl QrDisplay {
    pub fn new(url: &str, picker: Picker) -> Result<Self, qrcode::types::QrError> {
        let grid = ModuleGrid::new(url)?;
        let mode = match picker.protocol_type() {
            ProtocolType::Halfblocks => Mode::Blocks { grid },
            _ => Mode::Graphics {
                grid,
                protocol: None,
            },
        };
        Ok(Self {
            url: url.to_owned(),
            picker,
            mode,
        })
    }

    /// Whether this display already holds `url` (cache reuse).
    pub fn matches(&self, url: &str) -> bool {
        self.url == url
    }

    fn graphics_protocol(
        &mut self,
        area: ratatui::layout::Size,
    ) -> Option<&mut ratatui_image::protocol::Protocol> {
        let Mode::Graphics { grid, protocol } = &mut self.mode else {
            return None;
        };
        let cell = self.picker.font_size();
        let cell = ratatui::layout::Size::new(cell.width, cell.height);
        let geometry = GraphicsGeometry::fit(grid.side(), cell, area)?;
        if protocol
            .as_ref()
            .is_some_and(|existing| existing.size() == geometry.cells())
        {
            return protocol.as_mut();
        }
        let image = grid.raster(geometry)?;
        let encoded = self
            .picker
            .new_protocol(image, geometry.cells(), Resize::Fit(None))
            .ok()?;
        *protocol = Some(encoded);
        protocol.as_mut()
    }

    fn block_geometry(
        &self,
        area: ratatui::layout::Size,
    ) -> Option<super::geometry::BlockGeometry> {
        let Mode::Blocks { grid } = &self.mode else {
            return None;
        };
        let cell = self.picker.font_size();
        let cell = ratatui::layout::Size::new(cell.width, cell.height);
        super::geometry::BlockGeometry::fit(grid.block_side(), cell, area)
    }

    /// Minimum cells needed for a lossless, un-stretched render.
    pub fn minimum_size(&self) -> ratatui::layout::Size {
        let cell = self.picker.font_size();
        let cell = ratatui::layout::Size::new(cell.width.max(1), cell.height.max(1));
        match &self.mode {
            Mode::Graphics { grid, .. } => GraphicsGeometry::minimum(grid.side(), cell),
            Mode::Blocks { grid } => BlockGeometry::minimum(grid.block_side(), cell),
        }
        .unwrap_or(ratatui::layout::Size::new(1, 1))
    }

    /// Render centered inside `area`; drawing follows the available space.
    /// Returns `false` only when the QR cannot be drawn at all (or graphics
    /// encoding failed, which falls back to blocks).
    pub fn render(&mut self, frame: &mut ratatui::Frame, area: Rect) -> bool {
        if area.width == 0 || area.height == 0 {
            return false;
        }
        let available = ratatui::layout::Size::new(area.width, area.height);
        if !matches!(self.mode, Mode::Blocks { .. }) {
            if let Some(protocol) = self.graphics_protocol(available) {
                let cells = protocol.size();
                let spot = Rect::new(
                    area.x + (area.width - cells.width) / 2,
                    area.y + (area.height - cells.height) / 2,
                    cells.width,
                    cells.height,
                );
                frame.render_widget(Image::new(protocol), spot);
                return true;
            }
            self.downgrade_to_blocks();
        }
        let Mode::Blocks { grid } = &self.mode else {
            return false;
        };
        let Some(geometry) = self.block_geometry(available) else {
            return false;
        };
        let cells = geometry.cells();
        let spot = Rect::new(
            area.x + (area.width - cells.width) / 2,
            area.y + (area.height - cells.height) / 2,
            cells.width,
            cells.height,
        );
        blocks::render(frame.buffer_mut(), spot, grid, geometry);
        true
    }

    /// Graphics encoding or sizing failed: keep the QR usable in this terminal.
    fn downgrade_to_blocks(&mut self) {
        if let Mode::Graphics { grid, .. } = &self.mode {
            let grid = grid.clone();
            self.mode = Mode::Blocks { grid };
        }
    }
}
