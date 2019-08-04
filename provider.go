package main

import (
	"context"
	"path"
	"time"

	"go.uber.org/zap"
)

// UpdateStats contains interesting statistics gathered while updating a
// repository.
type UpdateStats struct {
	BytesDownloaded int
	Duration        time.Duration
}

type Repo struct {
	Provider string
	Name     string
	URL      string
}

func (r Repo) Download(ctx context.Context, root string, logger *zap.Logger) (UpdateStats, error) {
	start := time.Now()
	dest := path.Join(root, r.Provider, r.Name)

	logger.Debug("Downloaded Repo",
		zap.String("dest", dest),
		zap.Duration("duration", time.Since(start)))

	return UpdateStats{}, nil
}
