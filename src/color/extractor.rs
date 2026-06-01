use std::path::Path;

pub enum ThemeMode {
    Light,
    Dark,
}

pub struct ColorPalette {
    pub mode: ThemeMode,
    /// 16 colors (usually hex strings like "#1E1E2E")
    pub colors: [String; 16], 
    pub background: String,
    pub foreground: String,
}

pub trait ColorExtractingStrategy {
    // input in image path 
    // output : dark mode or light mode , then 16 colors 
    fn extract(&self, image_path: &Path, mode: ThemeMode) -> Result<ColorPalette, String>;
}