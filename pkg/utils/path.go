package utils

import (
	"os"
	"path/filepath"
)

func AbsolutePath(path string) string {
	if filepath.IsAbs(path) {
		return path
	}

	exe, _ := os.Executable()
	exePath := filepath.Dir(exe)
	return filepath.Join(exePath, path)
}
