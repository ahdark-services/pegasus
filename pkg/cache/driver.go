package cache

import (
	"context"
	"time"

	"go.opentelemetry.io/otel"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/internal/cache")

type Driver interface {
	Get(ctx context.Context, key string) (interface{}, bool)
	MGet(ctx context.Context, keys []string) (map[string]interface{}, []string, error)
	GetEX(ctx context.Context, key string) (interface{}, time.Duration, error)
	Set(ctx context.Context, key string, value interface{}) error
	MSet(ctx context.Context, values map[string]interface{}) error
	SetEX(ctx context.Context, key string, value interface{}, expiration time.Duration) error
	Del(ctx context.Context, key string) error
	MDel(ctx context.Context, keys []string) error
	Keys(ctx context.Context, pattern string) ([]string, error)

	NewHashMap(ctx context.Context, key string) HashMap
	NewSet(ctx context.Context, key string) Set
}

type HashMap interface {
	Get(ctx context.Context, key string) (interface{}, bool)
	MGet(ctx context.Context, keys []string) (map[string]interface{}, error)
	Set(ctx context.Context, key string, value interface{}) error
	MSet(ctx context.Context, values map[string]interface{}) error
	Del(ctx context.Context, key string) error
	MDel(ctx context.Context, keys []string) error
	Keys(ctx context.Context, pattern string) ([]string, error)
}

type Set interface {
	Add(ctx context.Context, members ...interface{}) error
	Card(ctx context.Context) (int64, error)
	IsMember(ctx context.Context, member interface{}) (bool, error)
	Members(ctx context.Context) ([]interface{}, error)
	Pop(ctx context.Context) (interface{}, error)
	Remove(ctx context.Context, members ...interface{}) error
}
