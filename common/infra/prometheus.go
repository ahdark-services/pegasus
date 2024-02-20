package infra

import (
	"context"
	"errors"
	"github.com/prometheus/client_golang/prometheus/collectors"
	"net/http"

	promclient "github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

type PrometheusLogger struct {
	logger *zap.Logger
}

func (l *PrometheusLogger) Println(v ...interface{}) {
	l.logger.Sugar().Error(v...)
}

func NewPrometheusRegistry(ctx context.Context, lc fx.Lifecycle, vip *viper.Viper) (*promclient.Registry, error) {
	ctx, span := tracer.Start(ctx, "infra.NewPrometheusRegistry")
	defer span.End()

	registry := promclient.NewRegistry()

	svr := &http.Server{
		Addr: vip.GetString("observability.metric.reader.listen"),
		Handler: promhttp.HandlerFor(registry, promhttp.HandlerOpts{
			Registry:      registry,
			ErrorHandling: promhttp.HTTPErrorOnError,
			ErrorLog:      &PrometheusLogger{logger: otelzap.L().Named("prometheus")},
		}),
	}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "infra.NewPrometheusRegistry.OnStart")
			defer span.End()

			go func() {
				if err := svr.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
					span.RecordError(err)
					otelzap.L().Ctx(ctx).Panic("failed to start prometheus reader server", zap.Error(err))
				}
			}()

			return nil
		},
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "infra.NewPrometheusRegistry.OnStop")
			defer span.End()

			if err := svr.Shutdown(ctx); err != nil {
				span.RecordError(err)
				otelzap.L().Ctx(ctx).Error("failed to shutdown prometheus reader server", zap.Error(err))
				return err
			}

			return nil
		},
	})

	return registry, nil
}

func InvokePrometheusGoCollector(prom *promclient.Registry) {
	prom.MustRegister(collectors.NewGoCollector(collectors.WithGoCollectorRuntimeMetrics(collectors.MetricsAll)))
}
