use crate::error::Result;
use std::io::Write;

pub fn generate_and_save_qrcode(url: &str, filename: &str) -> Result<()> {
    use image::Luma;
    use qrcode::QrCode;
    use std::path::Path;

    let code = QrCode::new(url.as_bytes())?;

    let image = code
        .render::<Luma<u8>>()
        .quiet_zone(false)
        .min_dimensions(200, 200)
        .build();

    let path = Path::new(filename);
    image.save(path)?;

    Ok(())
}

pub fn print_qrcode_in_terminal(url: &str) -> Result<()> {
    use qrcode::QrCode;

    let code = QrCode::new(url.as_bytes())?;

    let string = code
        .render()
        .light_color("  ")
        .dark_color("██")
        .quiet_zone(false)
        .build();

    println!("{}", string);
    std::io::stdout().flush()?;
    Ok(())
}
