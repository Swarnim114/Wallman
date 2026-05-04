package color

// CustomExtractor implements our own minimal/custom color extraction logic.
type CustomExtractor struct{}

// Extract processes the image to find the dominant color.
func (c *CustomExtractor) Extract(imagePath string) (*Palette, error) {
	// Minimal mock implementation
	// TODO: Replace with actual image parsing and color math
	return &Palette{
		Dominant: "#FF0000",
	}, nil
}

// Name returns the identifier for this extractor.
func (c *CustomExtractor) Name() string {
	return "custom"
}
