package scanner

import (
	"os"
	"path/filepath"
	"strings"
)

func Scan(path string) ([]string, error) {

	entries, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	var images []string

	for _, entry := range entries {
		if !entry.IsDir() && isImage(entry.Name()) {
			images = append(images, filepath.Join(path, entry.Name()))
		}
	}
	return images, nil
}

func isImage(name string) bool {
	ext := strings.ToLower(filepath.Ext(name))

	if ext == ".jpg" ||
		ext == ".jpeg" ||
		ext == ".png" ||
		ext == ".webp" ||
		ext == ".bmp" ||
		ext == ".gif" ||
		ext == ".tiff" || ext == ".tif" ||
		ext == ".heic" || ext == ".heif" ||
		ext == ".avif" {
		return true
	}

	return false
}
