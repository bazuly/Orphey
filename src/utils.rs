use crate::error::Error;

pub fn now_ms() -> u64 {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    ms
}

// https://jsonplaceholder.typicode.com/posts/1

pub fn resolve_url(base: &str, path: &str) -> Result<String, Error> {
    let path = path.trim();

    if path.is_empty() {
        return Err(Error::Url("Url is empty!".into()));
    }

    if path.contains("://") {
        return Ok(path.to_string());
    }

    let base = base.trim().trim_end_matches("/");

    if base.is_empty() {
        return Err(Error::Url("Base url is empty!".into()));
    }

    let suffix = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    let url = format!("{base}{suffix}");
    Ok(url)
}
