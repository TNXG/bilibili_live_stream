//! QR module grid and black/white raster generation.

use image::{DynamicImage, ImageBuffer, Rgba};
use qrcode::{Color, QrCode};

use super::geometry::GraphicsGeometry;

const QUIET: u32 = 4;

/// The QR module grid including a quiet border, independent of display scale.
#[derive(Clone)]
pub struct ModuleGrid {
    side: u32,
    dark: Vec<bool>,
}

impl ModuleGrid {
    pub fn new(payload: &str) -> Result<Self, qrcode::types::QrError> {
        let code = QrCode::new(payload.as_bytes())?;
        let width = u32::try_from(code.width()).unwrap_or(0);
        let side = width + QUIET * 2;
        let mut dark = vec![false; (side * side) as usize];
        for y in 0..width {
            for x in 0..width {
                if code[(x as usize, y as usize)] == Color::Dark {
                    let index = ((y + QUIET) * side + x + QUIET) as usize;
                    dark[index] = true;
                }
            }
        }
        Ok(Self { side, dark })
    }

    /// Grid side including the quiet border.
    pub fn side(&self) -> u32 {
        self.side
    }

    /// Block-renderer side: rounded up to an even value so half-rows pair.
    pub fn block_side(&self) -> u32 {
        self.side + (self.side % 2)
    }

    pub fn dark(&self, x: u32, y: u32) -> bool {
        if x >= self.side || y >= self.side {
            return false;
        }
        self.dark[(y * self.side + x) as usize]
    }

    /// Square black/white raster for the graphics renderers.
    pub fn raster(&self, geometry: GraphicsGeometry) -> Option<DynamicImage> {
        let module = geometry.module_pixels();
        if module == 0
            || geometry
                .offset()
                .checked_mul(2)?
                .checked_add(module.checked_mul(self.side)?)?
                != geometry.pixels()
        {
            return None;
        }
        let pixels = geometry.pixels();
        let offset = geometry.offset();
        let mut buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(pixels, pixels);
        for y in 0..pixels {
            for x in 0..pixels {
                let dark = x >= offset
                    && y >= offset
                    && x < pixels - offset
                    && y < pixels - offset
                    && self.dark((x - offset) / module, (y - offset) / module);
                buffer.put_pixel(
                    x,
                    y,
                    if dark {
                        Rgba([0, 0, 0, 255])
                    } else {
                        Rgba([255, 255, 255, 255])
                    },
                );
            }
        }
        Some(DynamicImage::ImageRgba8(buffer))
    }
}
