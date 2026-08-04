
pub fn mask_rtmp_code(code: &str) -> String {
    if code.len() <= 10 {
        return code.to_string();
    }
    let prefix = &code[..6];
    let suffix = &code[code.len() - 4..];
    let masked_length = code.len() - 10;
    format!(
        "{}{}...{}",
        prefix,
        "*".repeat(masked_length.min(12)),
        suffix
    )
}
