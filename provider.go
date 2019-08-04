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

func Download(ctx context.Context, root string, repo Repo, logger *zap.Logger) (UpdateStats, error) {
	start := time.Now()
	dest := path.Join(root, repo.Provider, repo.Name)

	logger.Debug("Downloaded Repo",
		zap.String("dest", dest),
		zap.Duration("duration", time.Since(start)))

	panic("Unimplemented")
}
