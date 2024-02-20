package infra

import (
	"context"
	"fmt"
	"github.com/go-redis/redis_rate/v10"
	"github.com/go-redsync/redsync/v4"
	"github.com/go-redsync/redsync/v4/redis/goredis/v9"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/redis/go-redis/extra/redisotel/v9"
	"github.com/redis/go-redis/extra/redisprometheus/v9"
	"github.com/redis/go-redis/v9"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

func NewRedisClient(ctx context.Context, vip *viper.Viper) (redis.UniversalClient, error) {
	ctx, span := tracer.Start(ctx, "infra.NewRedisClient")
	defer span.End()

	var universalClient redis.UniversalClient
	switch vip.GetString("redis.mode") {
	case "standalone":
		client := redis.NewClient(&redis.Options{
			Network:  "tcp",
			Addr:     fmt.Sprintf("%s:%d", vip.GetString("redis.host"), vip.GetUint16("redis.port")),
			Username: vip.GetString("redis.username"),
			Password: vip.GetString("redis.password"),
			DB:       0,
		})

		universalClient = client
	case "replication":
		c := redis.NewClusterClient(&redis.ClusterOptions{
			Username:      vip.GetString("redis.username"),
			Password:      vip.GetString("redis.password"),
			RouteRandomly: true,
			ClusterSlots: func(_ context.Context) ([]redis.ClusterSlot, error) {
				return []redis.ClusterSlot{
					{
						Start: 0,
						End:   16383,
						Nodes: []redis.ClusterNode{
							{
								Addr: fmt.Sprintf("%s:%d", vip.GetString("redis.writer.host"), vip.GetUint16("redis.writer.port")),
							},
							{
								Addr: fmt.Sprintf("%s:%d", vip.GetString("redis.reader.host"), vip.GetUint16("redis.reader.port")),
							},
						},
					},
				}, nil
			},
		})

		c.ReloadState(ctx)

		universalClient = c
	default:
		otelzap.L().Ctx(ctx).Fatal("redis mode not supported", zap.String("mode", vip.GetString("redis.mode")))
		return nil, fmt.Errorf("redis mode %s not supported", vip.GetString("redis.mode"))
	}

	return universalClient, nil
}

func InjectRedisObservability(ctx context.Context, rdb redis.UniversalClient, vip *viper.Viper, prom *prometheus.Registry) (redis.UniversalClient, error) {
	ctx, span := tracer.Start(ctx, "infra.InjectRedisObservability")
	defer span.End()

	// Enable tracing instrumentation.
	if err := redisotel.InstrumentTracing(rdb); err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to inject tracer to redis", zap.Error(err))
		return nil, err
	}

	// Enable metrics instrumentation.
	if err := redisotel.InstrumentMetrics(rdb); err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to inject metrics to redis", zap.Error(err))
		return nil, err
	}

	collector := redisprometheus.NewCollector(
		vip.GetString("redis.metrics.namespace"),
		vip.GetString("redis.metrics.subsystem"),
		rdb,
	)
	if err := prom.Register(collector); err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to register redis metrics collector", zap.Error(err))
		return nil, err
	}

	return rdb, nil
}

func NewRedSync(ctx context.Context, rdb redis.UniversalClient) *redsync.Redsync {
	ctx, span := tracer.Start(ctx, "infra.NewRedSync")
	defer span.End()

	pool := goredis.NewPool(rdb)
	mutex := redsync.New(pool)

	return mutex
}

func NewRedisRateLimiter(ctx context.Context, rdb redis.UniversalClient) *redis_rate.Limiter {
	ctx, span := tracer.Start(ctx, "infra.NewRedisRateLimiter")
	defer span.End()

	return redis_rate.NewLimiter(rdb)
}
