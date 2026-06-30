// Package handler contains HTTP handlers for the REST API.
//
// File naming convention:
//   admin_*.go  — 运营后台 handler（需 Admin JWT + RBAC 权限）
//   *_test.go   — 测试
//   其余 *.go    — 用户端 / 公开端点 handler
//
// Root-level files (router.go, deps.go, ws.go, health.go) are shared infrastructure.
package handler

import (
	"github.com/redis/go-redis/v9"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/config"
	"minibili/internal/pkg/iplocate"
	"minibili/internal/pkg/jwttoken"
	"minibili/internal/pkg/sensitive"
	"minibili/internal/queue"
	"minibili/internal/search"
	"minibili/internal/service"
	"minibili/internal/storage"
	"minibili/internal/ws"
)

// Dependencies are shared across HTTP handlers.
type 	Dependencies struct {
	Cfg          *config.C
	DB           *gorm.DB
	Redis        *redis.Client
	Log          *zap.Logger
	Hub          *ws.Hub
	ChatHub      *ws.ChatHub
	JWT          *jwttoken.Manager
	Sens         *sensitive.Filter
	OSS          storage.FileStorager
	MQ           queue.TranscodePublisher
	ES           *search.Client
	Play         *service.PlayCounter
	SearchHot    *service.SearchHotRecorder
	DanmakuRelay *service.DanmakuRelay
	IPLocate     *iplocate.Searcher
	Agent        *service.AgentService
	Svcs         *service.Services
	Feed         *service.FeedService
}

// API exposes HTTP handlers.
type API struct {
	*Dependencies
}
