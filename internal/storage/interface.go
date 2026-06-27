package storage

import (
	"context"
	"io"
)

// FileStorager is the abstract storage interface that both OSS and local
// filesystem backends satisfy. Handlers depend on this interface instead of
// the concrete OSS type, so the system works without any cloud storage.
type FileStorager interface {
	UploadFile(objectKey, localPath string) error
	UploadReader(objectKey string, r io.Reader) error
	DeleteObject(objectKey string) error
	DeleteObjects(objectKeys []string) error
	Ping(ctx context.Context) (latencyMs int64, err error)
}
