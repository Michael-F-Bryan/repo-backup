package main

import (
	"context"

	"go.uber.org/zap"
)

func main() {
	logger := startLogger()

	cfg := Github{
		APIKey: "450ef93b9020581b54dc31d9019fc3d3a89dbc8a",
	}

	count := 0
	for repo := range FetchGithubRepos(context.Background(), cfg, logger) {
		logger.Info("Found a repo", zap.Any("repo", repo))
		count++
	}

	logger.Info("Found all repos", zap.Int("count", count))
}

func startLogger() *zap.Logger {
	cfg := zap.NewDevelopmentConfig()
	cfg.Level.SetLevel(zap.DebugLevel)
	logger, _ := cfg.Build()

	return logger
}
