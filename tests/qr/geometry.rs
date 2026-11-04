use bili_live::qr::{BlockGeometry, GraphicsGeometry};
use ratatui::layout::Size;

#[test]
fn graphics_use_pixel_square_not_equal_cell_counts() {
    let geometry = GraphicsGeometry::fit(29, Size::new(10, 20), Size::new(80, 24)).unwrap();
    assert_eq!(geometry.cells(), Size::new(48, 24));
    assert_eq!(geometry.pixels(), 480);
    assert_eq!(geometry.module_pixels(), 16);
    assert_eq!(geometry.offset(), 8);
}

#[test]
fn graphics_round_to_cell_lcm_without_fractional_modules() {
    let geometry = GraphicsGeometry::fit(29, Size::new(9, 17), Size::new(80, 24)).unwrap();
    assert_eq!(geometry.cells(), Size::new(34, 18));
    assert_eq!(geometry.pixels(), 306);
    assert_eq!(geometry.module_pixels(), 10);
    assert_eq!(geometry.offset(), 8);
    assert!(GraphicsGeometry::fit(29, Size::new(9, 17), Size::new(16, 24)).is_none());
}

#[test]
fn halfblocks_keep_modules_square_for_nonstandard_fonts() {
    for (font, expected, columns, halfrows) in [
        (Size::new(10, 20), Size::new(30, 15), 1, 1),
        (Size::new(8, 12), Size::new(90, 60), 3, 4),
        (Size::new(9, 17), Size::new(510, 270), 17, 18),
    ] {
        let geometry = BlockGeometry::fit(30, font, expected).unwrap();
        assert_eq!(geometry.cells(), expected);
        assert_eq!(geometry.module_columns(), columns);
        assert_eq!(geometry.module_halfrows(), halfrows);
        assert_eq!(
            columns * u32::from(font.width) * 2,
            halfrows * u32::from(font.height)
        );
        assert!(
            BlockGeometry::fit(30, font, Size::new(expected.width - 1, expected.height)).is_none()
        );
    }
}

#[test]
fn geometry_scales_in_integer_steps_and_rejects_unusable_sizes() {
    let geometry = BlockGeometry::fit(30, Size::new(10, 20), Size::new(80, 40)).unwrap();
    assert_eq!(geometry.cells(), Size::new(60, 30));
    assert_eq!(geometry.module_columns(), 2);
    assert_eq!(geometry.module_halfrows(), 2);
    for side in [0, u32::MAX] {
        assert!(GraphicsGeometry::fit(side, Size::new(10, 20), Size::new(80, 24)).is_none());
        assert!(BlockGeometry::fit(side, Size::new(10, 20), Size::new(80, 24)).is_none());
    }
    assert!(BlockGeometry::fit(29, Size::new(10, 20), Size::new(80, 24)).is_none());
    assert!(GraphicsGeometry::fit(29, Size::new(0, 20), Size::new(80, 24)).is_none());
    assert!(
        GraphicsGeometry::fit(
            29,
            Size::new(u16::MAX, u16::MAX - 1),
            Size::new(u16::MAX, u16::MAX)
        )
        .is_none()
    );
    assert!(BlockGeometry::fit(30, Size::new(10, 20), Size::new(0, 0)).is_none());
}

#[test]
fn graphics_bound_allocations_even_for_maximum_terminal_dimensions() {
    let geometry =
        GraphicsGeometry::fit(29, Size::new(10, 20), Size::new(u16::MAX, u16::MAX)).unwrap();
    assert!(geometry.pixels() <= 2048);
    assert_eq!(geometry.cells().width * 10, geometry.cells().height * 20);
}
