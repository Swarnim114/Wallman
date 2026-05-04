package color

import "fmt"

// Palette represents the unified color structure returned by any extractor.
type Palette struct {
	Dominant string
	// Add more standard fields here (e.g., Background, Foreground, etc.)
}

// Extractor is the interface (Strategy) that all color extraction methods must implement.
type Extractor interface {
	Extract(imagePath string) (*Palette, error)
	Name() string
}

// GetExtractor is a Factory function to instantiate the correct extractor.
func GetExtractor(name string) (Extractor, error) {
	switch name {
	case "custom":
		return &CustomExtractor{}, nil
	case "matugen":
		return &MatugenExtractor{}, nil
	default:
		return nil, fmt.Errorf("unknown extractor: %s", name)
	}
}
