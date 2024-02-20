package infra

import (
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/internal/infra")

func Module() fx.Option {
	return fx.Module(
		"internal.infra",
		fx.Provide(NewCacheDriver),
		fx.Provide(NewEtcdClient),
		fx.Provide(NewAMQPConn),
		fx.Provide(NewAMQPChannel),
		fx.Provide(NewPrometheusRegistry),
		fx.Invoke(InvokePrometheusGoCollector),
		fx.Provide(NewRedisClient),
		fx.Decorate(InjectRedisObservability),
		fx.Provide(NewRedSync),
		fx.Provide(NewRedisRateLimiter),
	)
}
