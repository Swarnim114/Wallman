use std::path::Path;

pub fn has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}