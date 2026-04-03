pub fn get_query_string(name: &str, url: &str) -> String {
    let pairs: Vec<&str> = url.split('?').nth(1).unwrap_or("").split('&').collect();

    for pair in pairs {
        let mut parts = pair.split('=');
        if let Some(key) = parts.next()
            && key == name
        {
            return parts.next().unwrap_or("").to_string();
        }
    }
    String::new()
}

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
