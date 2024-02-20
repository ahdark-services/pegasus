package cache

import (
	"context"
	"time"

	bs "github.com/AH-dark/bytestring/v2"
	"github.com/redis/go-redis/v9"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
)

const redisScanCount = 16

type redisDriver struct {
	client redis.UniversalClient
}

func NewRedisDriver(client redis.UniversalClient) Driver {
	return &redisDriver{client: client}
}

func (rdb *redisDriver) Get(ctx context.Context, key string) (interface{}, bool) {
	ctx, span := tracer.Start(ctx, "RedisDriver.Get", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	res, err := rdb.client.Get(ctx, key).Bytes()
	if err != nil {
		span.RecordError(err)
		return nil, false
	}

	item, err := decodeRedisItem(res)
	if err != nil {
		span.RecordError(err)
		return nil, false
	}

	return item.Value, true
}

func (rdb *redisDriver) MGet(ctx context.Context, keys []string) (map[string]interface{}, []string, error) {
	ctx, span := tracer.Start(ctx, "RedisDriver.MGet", trace.WithAttributes(
		attribute.Int("count", len(keys)),
	))
	defer span.End()

	res, err := rdb.client.MGet(ctx, keys...).Result()
	if err != nil {
		span.RecordError(err)
		return nil, keys, err
	}

	items := make(map[string]interface{}, len(res))
	missed := make([]string, 0, len(res))
	for i := range res {
		if res[i] == nil {
			missed = append(missed, keys[i])
			continue
		}

		item, err := decodeRedisItem(bs.S2B(res[i].(string)))
		if err != nil {
			span.RecordError(err)
			return nil, keys, err
		}

		items[keys[i]] = item.Value
	}

	return items, missed, nil
}

func (rdb *redisDriver) GetEX(ctx context.Context, key string) (interface{}, time.Duration, error) {
	ctx, span := tracer.Start(ctx, "RedisDriver.GetEX", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	res, err := rdb.client.Get(ctx, key).Bytes()
	if err != nil {
		span.RecordError(err)
		return nil, 0, err
	}

	item, err := decodeRedisItem(res)
	if err != nil {
		span.RecordError(err)
		return nil, 0, err
	}

	exp, err := rdb.client.TTL(ctx, key).Result()
	if err != nil {
		span.RecordError(err)
		return nil, 0, err
	}

	return item.Value, exp, nil
}

func (rdb *redisDriver) Set(ctx context.Context, key string, value interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisDriver.Set", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	item := redisItem{Value: value}
	gob, err := encodeRedisItem(item)
	if err != nil {
		span.RecordError(err)
		return err
	}

	return rdb.client.Set(ctx, key, bs.B2S(gob), 0).Err()
}

func (rdb *redisDriver) MSet(ctx context.Context, values map[string]interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisDriver.MSet", trace.WithAttributes(
		attribute.Int("count", len(values)),
	))
	defer span.End()

	args := make([]interface{}, 0, len(values)*2)
	for k := range values {
		gob, err := encodeRedisItem(redisItem{Value: values[k]})
		if err != nil {
			span.RecordError(err)
			return err
		}

		args = append(args, k, bs.B2S(gob))
	}

	return rdb.client.MSet(ctx, args...).Err()
}

func (rdb *redisDriver) SetEX(ctx context.Context, key string, value interface{}, expiration time.Duration) error {
	ctx, span := tracer.Start(ctx, "RedisDriver.SetEX", trace.WithAttributes(
		attribute.String("key", key),
		attribute.Int64("expiration", expiration.Milliseconds()),
	))
	defer span.End()

	item := redisItem{Value: value}
	gob, err := encodeRedisItem(item)
	if err != nil {
		span.RecordError(err)
		return err
	}

	return rdb.client.Set(ctx, key, bs.B2S(gob), expiration).Err()
}

func (rdb *redisDriver) Del(ctx context.Context, key string) error {
	ctx, span := tracer.Start(ctx, "RedisDriver.Del", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	return rdb.client.Del(ctx, key).Err()
}

func (rdb *redisDriver) MDel(ctx context.Context, keys []string) error {
	ctx, span := tracer.Start(ctx, "RedisDriver.MDel", trace.WithAttributes(
		attribute.StringSlice("keys", keys),
		attribute.Int("count", len(keys)),
	))
	defer span.End()

	return rdb.client.Del(ctx, keys...).Err()
}

func (rdb *redisDriver) Keys(ctx context.Context, pattern string) ([]string, error) {
	ctx, span := tracer.Start(ctx, "RedisDriver.Keys", trace.WithAttributes(
		attribute.String("pattern", pattern),
	))
	defer span.End()

	var keys []string
	var pre uint64
	for {
		var res []string
		var err error
		res, cur, err := rdb.client.Scan(ctx, pre, pattern, redisScanCount).Result()
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		keys = append(keys, res...)
		if cur == 0 {
			break
		}
		pre = cur
	}

	return keys, nil
}

func (rdb *redisDriver) NewHashMap(ctx context.Context, key string) HashMap {
	ctx, span := tracer.Start(ctx, "RedisDriver.NewHashMap", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	return &redisHashMap{client: rdb.client, key: key}
}

func (rdb *redisDriver) NewSet(ctx context.Context, key string) Set {
	ctx, span := tracer.Start(ctx, "RedisDriver.NewSet", trace.WithAttributes(
		attribute.String("key", key),
	))
	defer span.End()

	return &redisSet{client: rdb.client, key: key}
}
