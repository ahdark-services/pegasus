package settings

import (
	"context"
	"fmt"
	"net/url"
	"strconv"
	"time"

	"github.com/dgraph-io/ristretto"
	"github.com/pkg/errors"
	"github.com/shopspring/decimal"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/namespace"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/internal/settings")

type implement struct {
	fx.In  `ignore-unexported:"true"`
	Client *clientv3.Client

	kv      clientv3.KV
	watcher clientv3.Watcher
	store   *ristretto.Cache
}

const storeDuration = 15 * time.Minute

func NewSettings(s implement, lc fx.Lifecycle) (Settings, error) {
	s.kv = namespace.NewKV(s.Client.KV, "settings:")
	s.watcher = namespace.NewWatcher(s.Client.Watcher, "settings:")

	var err error
	s.store, err = ristretto.NewCache(&ristretto.Config{
		NumCounters: 1e6,     // number of keys to track frequency of (100K).
		MaxCost:     1 << 30, // maximum cost of cache (1GB).
		BufferItems: 64,      // number of keys per Get buffer.
	})
	if err != nil {
		return nil, errors.Wrap(err, "failed to create cache")
	}

	lc.Append(fx.Hook{
		OnStop: func(context.Context) error {
			s.store.Close()
			return nil
		},
	})

	return &s, nil
}

func (svc *implement) handleEvent(key string, watchResp clientv3.WatchResponse) {
	ctx, span := tracer.Start(context.Background(), "Settings.handleEvent", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	if watchResp.Err() != nil {
		otelzap.L().Ctx(ctx).Error("failed to watch for changes", zap.Error(watchResp.Err()), zap.String("key", key))
		return
	}

	if watchResp.Events == nil || len(watchResp.Events) == 0 {
		otelzap.L().Ctx(ctx).Warn("empty watch response", zap.String("key", key))
		return
	}

	event := watchResp.Events[len(watchResp.Events)-1]
	otelzap.L().Ctx(ctx).Debug("setting changed", zap.String("key", key), zap.String("value", bs.B2S(event.Kv.Value)))
	switch event.Type {
	case clientv3.EventTypePut:
		svc.store.SetWithTTL(key, bs.B2S(event.Kv.Value), 0, storeDuration)
	case clientv3.EventTypeDelete:
		svc.store.Del(key)
	default:
		otelzap.L().Ctx(ctx).Warn("unknown event type", zap.String("key", key), zap.String("type", event.Type.String()))
	}
}

func (svc *implement) getSetting(ctx context.Context, key string) (string, error) {
	ctx, span := tracer.Start(ctx, "Settings.getSetting", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	// Fetching from cache
	span.AddEvent("fetching-from-cache")
	if v, ok := svc.store.Get(key); ok {
		if s, ok := v.(string); ok {
			otelzap.L().Ctx(ctx).Debug("setting found in cache", zap.String("key", key), zap.String("value", s))
			return s, nil
		}
	}

	// Fetching from etcd
	span.AddEvent("fetching-from-etcd")
	resp, err := svc.kv.Get(ctx, key, clientv3.WithLastRev()...)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting from etcd", zap.Error(err), zap.String("key", key))
		return "", errors.Wrap(err, "failed to get setting from etcd")
	}

	// If key is not present in etcd
	if resp.Count == 0 {
		span.SetStatus(codes.Error, "key not found")
		otelzap.L().Ctx(ctx).Error("key not found", zap.Error(err), zap.String("key", key))
		return "", errors.New("key not found")
	}

	// Store in cache
	span.AddEvent("store-in-cache")
	svc.store.SetWithTTL(key, bs.B2S(resp.Kvs[0].Value), 0, storeDuration)

	// Watch for changes
	span.AddEvent("watch-for-changes")
	go func(key string) {
		defer svc.store.Del(key)

		for watchResp := range svc.watcher.Watch(ctx, key) {
			svc.handleEvent(key, watchResp)
		}
	}(key)

	return bs.B2S(resp.Kvs[0].Value), nil
}

func (svc *implement) GetString(ctx context.Context, key string) (string, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetString", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return "", errors.Wrap(err, "failed to get setting")
	}

	return s, nil
}

func (svc *implement) GetInt64(ctx context.Context, key string) (int64, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetInt64", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to get setting")
	}

	i, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to parse setting")
	}

	return i, nil
}

func (svc *implement) GetUint64(ctx context.Context, key string) (uint64, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetUint64", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to get setting")
	}

	i, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to parse setting")
	}

	return i, nil
}

func (svc *implement) GetBool(ctx context.Context, key string) (bool, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetBool", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return false, errors.WithMessage(err, "failed to get setting")
	}

	b, err := strconv.ParseBool(s)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return false, errors.WithMessage(err, "failed to parse setting")
	}

	return b, nil
}

func (svc *implement) GetTimeDuration(ctx context.Context, key string) (time.Duration, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetTimeDuration", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to get setting")
	}

	d, err := time.ParseDuration(s)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return 0, errors.WithMessage(err, "failed to parse setting")
	}

	return d, nil
}

func (svc *implement) GetDecimal(ctx context.Context, key string) (decimal.Decimal, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetDecimal", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return decimal.Zero, errors.WithMessage(err, "failed to get setting")
	}

	d, err := decimal.NewFromString(s)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return decimal.Zero, errors.WithMessage(err, "failed to parse setting")
	}

	return d, nil
}

func (svc *implement) GetUrl(ctx context.Context, key string) (*url.URL, error) {
	ctx, span := tracer.Start(ctx, "Settings.GetUrl", trace.WithAttributes(
		attribute.String("setting.key", key),
	))
	defer span.End()

	s, err := svc.getSetting(ctx, key)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
		return nil, errors.WithMessage(err, "failed to get setting")
	}

	u, err := url.Parse(s)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to parse setting", zap.Error(err), zap.String("key", key))
		return nil, errors.WithMessage(err, "failed to parse setting")
	}

	return u, nil
}

func (svc *implement) ListStrings(ctx context.Context, keys []string) (map[string]string, error) {
	ctx, span := tracer.Start(ctx, "Settings.ListStrings", trace.WithAttributes(
		attribute.StringSlice("setting.keys", keys),
	))
	defer span.End()

	data := make(map[string]string, len(keys))
	for _, key := range keys {
		s, err := svc.getSetting(ctx, key)
		if err != nil {
			span.RecordError(err)
			otelzap.L().Ctx(ctx).Error("failed to get setting", zap.Error(err), zap.String("key", key))
			return nil, fmt.Errorf("failed to get setting: %w, key %s", err, key)
		}

		data[key] = s
		otelzap.L().Ctx(ctx).Debug("setting found", zap.String("key", key), zap.String("value", s))
	}

	return data, nil
}

func (svc *implement) SaveString(ctx context.Context, key string, value string) error {
	ctx, span := tracer.Start(ctx, "Settings.SaveString")
	defer span.End()

	resp, err := svc.kv.Put(ctx, key, value)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to update setting", zap.Error(err), zap.String("key", key), zap.String("value", value))
		return fmt.Errorf("failed to update setting: %w", err)
	}

	otelzap.L().Ctx(ctx).Debug("setting updated", zap.String("key", key), zap.String("value", value), zap.Int64("revision", resp.Header.Revision))

	return nil
}
