package cache

import (
	"context"

	bs "github.com/AH-dark/bytestring/v2"
	"github.com/redis/go-redis/v9"
)

type redisSet struct {
	client redis.UniversalClient
	key    string
}

var _ Set = (*redisSet)(nil)

func (rs *redisSet) Add(ctx context.Context, members ...interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisSet.Add")
	defer span.End()

	args := make([]interface{}, 0, len(members))
	for _, v := range members {
		gob, err := encodeRedisItem(redisItem{Value: v})
		if err != nil {
			span.RecordError(err)
			return err
		}

		args = append(args, gob)
	}

	if _, err := rs.client.SAdd(ctx, rs.key, args...).Result(); err != nil {
		span.RecordError(err)
		return err
	}

	return nil
}

func (rs *redisSet) Card(ctx context.Context) (int64, error) {
	ctx, span := tracer.Start(ctx, "RedisSet.Card")
	defer span.End()

	return rs.client.SCard(ctx, rs.key).Result()
}

func (rs *redisSet) IsMember(ctx context.Context, member interface{}) (bool, error) {
	ctx, span := tracer.Start(ctx, "RedisSet.IsMember")
	defer span.End()

	gob, err := encodeRedisItem(redisItem{Value: member})
	if err != nil {
		span.RecordError(err)
		return false, err
	}

	return rs.client.SIsMember(ctx, rs.key, gob).Result()
}

func (rs *redisSet) Members(ctx context.Context) ([]interface{}, error) {
	ctx, span := tracer.Start(ctx, "RedisSet.Members")
	defer span.End()

	res, err := rs.client.SMembers(ctx, rs.key).Result()
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	items := make([]interface{}, 0, len(res))
	for _, v := range res {
		item, err := decodeRedisItem(bs.S2B(v))
		if err != nil {
			span.RecordError(err)
			return nil, err
		}

		items = append(items, item.Value)
	}

	return items, nil
}

func (rs *redisSet) Pop(ctx context.Context) (interface{}, error) {
	ctx, span := tracer.Start(ctx, "RedisSet.Pop")
	defer span.End()

	res, err := rs.client.SPop(ctx, rs.key).Result()
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	item, err := decodeRedisItem(bs.S2B(res))
	if err != nil {
		span.RecordError(err)
		return nil, err
	}

	return item.Value, nil
}

func (rs *redisSet) Remove(ctx context.Context, members ...interface{}) error {
	ctx, span := tracer.Start(ctx, "RedisSet.Remove")
	defer span.End()

	args := make([]interface{}, 0, len(members))
	for _, v := range members {
		gob, err := encodeRedisItem(redisItem{Value: v})
		if err != nil {
			span.RecordError(err)
			return err
		}

		args = append(args, gob)
	}

	if _, err := rs.client.SRem(ctx, rs.key, args...).Result(); err != nil {
		span.RecordError(err)
		return err
	}

	return nil
}
