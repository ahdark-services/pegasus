package cache

import (
	"context"

	bs "github.com/AH-dark/bytestring/v2"
	"github.com/redis/go-redis/v9"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
)

type redisHashMap struct {
	client redis.UniversalClient
	key    string
}

var _ HashMap = (*redisHashMap)(nil)

func (rhm *redisHashMap) Get(ctx context.Context, key string) (interface{}, bool) {
	ctx, span := tracer.Start(ctx, "RedisHashMap.Get")
	defer span.End()

	res, err := rhm.client.HGet(ctx, rhm.key, key).Bytes()
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

func (rhm *redisHashMap) MGet(ctx context.Context, keys []string) (map[string]interface{}, error) {
	ctx, span := tracer.Start(ctx, "RedisHashMap.MGet")
	defer span.End()

	res, err := rhm.client.HMGet(ctx, rhm.key, keys...).Result()
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	items := make(map[string]interface{}, len(res))
	for i, v := range res {
		if v == nil {
			continue
		}

		item, err := decodeRedisItem(bs.S2B(v.(string)))
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		items[keys[i]] = item.Value
	}

	return items, nil
}

func (rhm *redisHashMap) Set(ctx context.Context, key string, value interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisHashMap.Set")
	defer span.End()

	res, err := encodeRedisItem(redisItem{Value: value})
	if err != nil {
		span.RecordError(err)
		return err
	}

	return rhm.client.HSet(ctx, rhm.key, key, res).Err()
}

func (rhm *redisHashMap) MSet(ctx context.Context, values map[string]interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisHashMap.MSet")
	defer span.End()

	args := make([]interface{}, 0, len(values)*2)
	for k := range values {
		res, err := encodeRedisItem(redisItem{Value: values[k]})
		if err != nil {
			span.RecordError(err)
			return err
		}

		args = append(args, k, res)
	}

	return rhm.client.HMSet(ctx, rhm.key, args...).Err()
}

func (rhm *redisHashMap) Del(ctx context.Context, key string) error {
	ctx, span := tracer.Start(ctx, "RedisHashMap.Del")
	defer span.End()

	return rhm.client.HDel(ctx, rhm.key, key).Err()
}

func (rhm *redisHashMap) MDel(ctx context.Context, keys []string) error {
	ctx, span := tracer.Start(ctx, "RedisHashMap.MDel")
	defer span.End()

	return rhm.client.HDel(ctx, rhm.key, keys...).Err()
}

func (rhm *redisHashMap) Keys(ctx context.Context, pattern string) ([]string, error) {
	ctx, span := tracer.Start(ctx, "RedisHashMap.Keys", trace.WithAttributes(
		attribute.String("pattern", pattern),
	))
	defer span.End()

	var keys []string
	var pre uint64
	for {
		res, cur, err := rhm.client.HScan(ctx, rhm.key, pre, pattern, 100).Result()
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
