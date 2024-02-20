package utils

import (
	"os"
	"path/filepath"
)

func CreateIfNotExist(path string) (*os.File, error) {
	path = AbsolutePath(path)

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, err
	}

	return os.OpenFile(path, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
}
