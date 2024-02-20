package settings

import (
	"context"
	"net/url"
	"time"

	"github.com/shopspring/decimal"
)

type Settings interface {
	GetString(ctx context.Context, key string) (string, error)
	GetInt64(ctx context.Context, key string) (int64, error)
	GetUint64(ctx context.Context, key string) (uint64, error)
	GetBool(ctx context.Context, key string) (bool, error)
	GetTimeDuration(ctx context.Context, key string) (time.Duration, error)
	GetDecimal(ctx context.Context, key string) (decimal.Decimal, error)
	GetUrl(ctx context.Context, key string) (*url.URL, error)

	ListStrings(ctx context.Context, keys []string) (map[string]string, error)

	SaveString(ctx context.Context, key string, value string) error
}
