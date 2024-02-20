package infra

import (
	cache2 "github.com/ahdark-services/pegasus/pkg/cache"
)

func NewCacheDriver(redisClient redis.UniversalClient) cache2.Driver {
	return cache2.NewRedisDriver(redisClient)
}
