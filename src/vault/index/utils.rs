use std::path::Path;
use std::collections::HashSet;


// only img extensions are allowed
// check the file path and verifies it against a set of extensions
pub fn has_matching_extension(path: &Path, extensions: &[String]) -> bool {
    // If the extensions list is empty, allow everything
    if extensions.is_empty() {
        return true;
    }

    // Get the extension safely (returns Some(&OsStr) or None)
    if let Some(os_ext) = path.extension() {
        
        // Convert the OsStr system string into a normal Rust string slice (&str)
        if let Some(ext_str) = os_ext.to_str() {
            
            //  Loop through your allowed list and check for a match
            for allowed in extensions {
                if allowed.eq_ignore_ascii_case(ext_str) {
                    return true; // Match found!
                }
            }
            
        }
    }

    // If the file has no extension, or it didn't match anything in the loop
    false
}



/// Checks if a given extension string is a valid image/wallpaper format.
pub fn is_image_extension(ext: &str) -> bool {
    // 1. Create our set of valid wallpaper/image extensions
    let valid_images: HashSet<&str> = HashSet::from([
        "jpg", "jpeg", "png", "bmp", "tiff", "tif",
        "webp", "avif", "heic", "heif", "jxl",
        "mp4", "gif", "webm", "apng",
        "svg", "ai", "eps", "pdf", "psd"
    ]);

    // 2. Normalize the input to lowercase so "PNG" or "Jpg" still match perfectly
    let normalized = ext.to_ascii_lowercase();

    // 3. Check if the set contains the extension string slice
    valid_images.contains(normalized.as_str())
}