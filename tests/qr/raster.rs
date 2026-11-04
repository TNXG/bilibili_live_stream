use bili_live::qr::{GraphicsGeometry, ModuleGrid};
use qrcode::{Color, QrCode};
use ratatui::layout::Size;

#[test]
fn grid_preserves_qrcode_modules_and_four_white_border_modules() {
    let grid = ModuleGrid::new("A").unwrap();
    let code = QrCode::new("A").unwrap();
    assert_eq!(grid.side(), 29);
    assert_eq!(grid.block_side(), 30);
    for y in 0..29 {
        for x in 0..29 {
            let expected = if (4..25).contains(&x) && (4..25).contains(&y) {
                code[((x - 4) as usize, (y - 4) as usize)] == Color::Dark
            } else {
                false
            };
            assert_eq!(grid.dark(x, y), expected, "at {x},{y}");
        }
    }
    assert!(!grid.dark(29, 29));
    assert!(!grid.dark(u32::MAX, u32::MAX));
}

#[test]
fn raster_is_square_opaque_blackwhite_and_uniform_integer_modules() {
    let grid = ModuleGrid::new("A").unwrap();
    let geometry = GraphicsGeometry::fit(29, Size::new(10, 20), Size::new(80, 24)).unwrap();
    let image = grid.raster(geometry).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (480, 480));
    for (x, y, pixel) in image.enumerate_pixels() {
        let dark =
            (8..472).contains(&x) && (8..472).contains(&y) && grid.dark((x - 8) / 16, (y - 8) / 16);
        assert_eq!(pixel.0, if dark { [0, 0, 0, 255] } else { [255; 4] });
    }
}

#[test]
fn raster_rejects_geometry_for_a_different_grid() {
    let grid = ModuleGrid::new("A").unwrap();
    let geometry = GraphicsGeometry::fit(21, Size::new(10, 20), Size::new(80, 24)).unwrap();
    assert!(grid.raster(geometry).is_none());
}
