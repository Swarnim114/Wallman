package color

// MatugenExtractor implements color extraction via the external 'matugen' CLI.
type MatugenExtractor struct{}

// Extract shells out to 'matugen' and parses its output.
func (m *MatugenExtractor) Extract(imagePath string) (*Palette, error) {
	// Minimal mock implementation
	// TODO: Replace with os/exec call to `matugen image <imagePath> -j` and parse JSON
	return &Palette{
		Dominant: "#00FF00",
	}, nil
}

// Name returns the identifier for this extractor.
func (m *MatugenExtractor) Name() string {
	return "matugen"
}
