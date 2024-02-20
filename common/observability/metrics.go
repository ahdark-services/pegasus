package observability

import (
	"context"
	"fmt"
	"go.opentelemetry.io/otel"

	promclient "github.com/prometheus/client_golang/prometheus"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel/exporters/prometheus"
	metricsdk "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	"go.uber.org/zap"
)

func NewMeterReader(ctx context.Context, vip *viper.Viper, registerer *promclient.Registry) (metricsdk.Reader, error) {
	ctx, span := tracer.Start(ctx, "observability.NewMeterReader")
	defer span.End()

	readerType := vip.GetString("observability.metric.reader.type")
	switch readerType {
	case "prometheus":
		exporter, err := prometheus.New(
			prometheus.WithNamespace(vip.GetString("namespace")),
			prometheus.WithRegisterer(registerer),
		)
		if err != nil {
			span.RecordError(err)
			otelzap.L().Ctx(ctx).Error("failed to create metric reader", zap.Error(err))
			return nil, err
		}

		return exporter, nil
	default:
		otelzap.L().Ctx(ctx).Fatal("unknown metric reader type", zap.String("type", readerType))
		return nil, fmt.Errorf("unknown metric reader type: %s", readerType)
	}
}

func NewMeterProvider(
	ctx context.Context,
	resource *resource.Resource,
	reader metricsdk.Reader,
) *metricsdk.MeterProvider {
	ctx, span := tracer.Start(ctx, "observability.NewMeterProvider")
	defer span.End()

	mp := metricsdk.NewMeterProvider(
		metricsdk.WithResource(resource),
		metricsdk.WithReader(reader),
	)

	return mp
}

func InitMeterProvider(ctx context.Context, mp *metricsdk.MeterProvider) {
	ctx, span := tracer.Start(ctx, "observability.InitMeterProvider")
	defer span.End()

	otel.SetMeterProvider(mp)
}
