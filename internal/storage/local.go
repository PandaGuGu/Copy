package storage

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Local stores files on the local filesystem. It is a drop-in replacement for
// OSS that requires zero configuration — everything stays inside the server.
type Local struct {
	BaseDir string // e.g. "data/uploads"
}

// NewLocal creates a Local storage rooted at baseDir (created if missing).
func NewLocal(baseDir string) (*Local, error) {
	if err := os.MkdirAll(baseDir, 0755); err != nil {
		return nil, fmt.Errorf("local storage: mkdir %s: %w", baseDir, err)
	}
	return &Local{BaseDir: baseDir}, nil
}

// path returns the full filesystem path for objectKey. Traversal (../) is rejected.
func (l *Local) path(objectKey string) (string, error) {
	key := strings.TrimPrefix(strings.TrimSpace(objectKey), "/")
	if key == "" {
		return "", fmt.Errorf("empty object key")
	}
	// Prevent directory traversal
	clean := filepath.Clean(key)
	if strings.Contains(clean, "..") {
		return "", fmt.Errorf("invalid object key: %s", objectKey)
	}
	return filepath.Join(l.BaseDir, clean), nil
}

// UploadFile copies a local file to objectKey.
func (l *Local) UploadFile(objectKey, localPath string) error {
	dst, err := l.path(objectKey)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(dst), 0755); err != nil {
		return err
	}
	src, err := os.Open(localPath)
	if err != nil {
		return err
	}
	defer src.Close()
	dstFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer dstFile.Close()
	if _, err := io.Copy(dstFile, src); err != nil {
		return err
	}
	return nil
}

// UploadReader writes the contents of r to objectKey.
func (l *Local) UploadReader(objectKey string, r io.Reader) error {
	dst, err := l.path(objectKey)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(dst), 0755); err != nil {
		return err
	}
	f, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer f.Close()
	if _, err := io.Copy(f, r); err != nil {
		return err
	}
	return nil
}

// DeleteObject removes one file. Missing files are silently ignored.
func (l *Local) DeleteObject(objectKey string) error {
	p, err := l.path(objectKey)
	if err != nil {
		return err
	}
	if err := os.Remove(p); err != nil && !os.IsNotExist(err) {
		return err
	}
	return nil
}

// DeleteObjects removes multiple files. Empty keys and missing files are skipped.
func (l *Local) DeleteObjects(objectKeys []string) error {
	for _, k := range objectKeys {
		_ = l.DeleteObject(k) // best-effort, ignore individual errors
	}
	return nil
}

// Ping checks that the storage directory exists and is writable (local is always fast).
func (l *Local) Ping(_ context.Context) (latencyMs int64, err error) {
	start := time.Now()
	testFile := filepath.Join(l.BaseDir, ".healthcheck")
	if err := os.WriteFile(testFile, []byte("1"), 0644); err != nil {
		return 0, err
	}
	_ = os.Remove(testFile)
	return time.Since(start).Milliseconds(), nil
}
