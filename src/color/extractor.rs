use std::path::Path;

pub enum ThemeMode {
    Light,
    Dark,
}

pub struct ColorPalette {
    pub mode: ThemeMode,
    /// 16 colors (usually hex strings like "#1E1E2E")
    pub colors: [String; 16],
    /// the main dark base color
    pub background: String,
    /// bg shifted lighter — used for sidebars, panels, surfaces
    pub secondary_background: String,
    /// the main light text color
    pub foreground: String,
    /// fg shifted dimmer — used for comments, subtext, inactive items
    pub secondary_foreground: String,
}

pub trait ColorExtractingStrategy {
    // input in image path 
    // output : dark mode or light mode , then 16 colors 
    fn extract(&self, image_path: &Path, mode: ThemeMode) -> Result<ColorPalette, String>;
}