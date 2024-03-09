package observability

import (
	"context"
	"fmt"

	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	tracesdk "go.opentelemetry.io/otel/sdk/trace"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/internal/observability")

func NewTraceExporter(ctx context.Context, vip *viper.Viper) (tracesdk.SpanExporter, error) {
	ctx, span := tracer.Start(ctx, "observability.NewTraceExporter")
	defer span.End()

	exporterType := vip.GetString("observability.trace.exporter.type")

	switch exporterType {
	case "otlp-grpc":
		opts := []otlptracegrpc.Option{
			otlptracegrpc.WithEndpoint(vip.GetString("observability.trace.exporter.endpoint")),
			otlptracegrpc.WithTimeout(vip.GetDuration("observability.trace.exporter.timeout")),
		}
		if vip.GetBool("observability.trace.exporter.insecure") {
			opts = append(opts, otlptracegrpc.WithInsecure())
		}

		exporter, err := otlptracegrpc.New(ctx, opts...)
		if err != nil {
			span.RecordError(err)
			otelzap.L().Ctx(ctx).Error("failed to create trace exporter", zap.Error(err))
			return nil, err
		}

		return exporter, nil
	case "otlp-http":
		opts := []otlptracegrpc.Option{
			otlptracegrpc.WithEndpoint(vip.GetString("observability.trace.exporter.endpoint")),
			otlptracegrpc.WithTimeout(vip.GetDuration("observability.trace.exporter.timeout")),
		}
		if vip.GetBool("observability.trace.exporter.insecure") {
			opts = append(opts, otlptracegrpc.WithInsecure())
		}

		exporter, err := otlptracegrpc.New(ctx, opts...)
		if err != nil {
			span.RecordError(err)
			otelzap.L().Ctx(ctx).Error("failed to create trace exporter", zap.Error(err))
			return nil, err
		}

		return exporter, nil
	default:
		otelzap.L().Ctx(ctx).Fatal("unknown trace exporter type", zap.String("type", exporterType))
		return nil, fmt.Errorf("unknown trace exporter type: %s", exporterType)
	}
}

func NewTraceProvider(
	ctx context.Context,
	lc fx.Lifecycle,
	vip *viper.Viper,
	resource *resource.Resource,
	exporter tracesdk.SpanExporter,
) *tracesdk.TracerProvider {
	ctx, span := tracer.Start(ctx, "observability.NewTraceProvider")
	defer span.End()

	tp := tracesdk.NewTracerProvider(
		tracesdk.WithResource(resource),
		tracesdk.WithBatcher(
			exporter,
			tracesdk.WithBatchTimeout(vip.GetDuration("observability.trace.batch_timeout")),
			tracesdk.WithMaxExportBatchSize(vip.GetInt("observability.trace.max_batch_size")),
			tracesdk.WithExportTimeout(vip.GetDuration("observability.trace.export_timeout")),
			tracesdk.WithMaxQueueSize(vip.GetInt("observability.trace.max_queue_size")),
		),
		tracesdk.WithSampler(tracesdk.ParentBased(tracesdk.TraceIDRatioBased(vip.GetFloat64("observability.trace.sampling_ratio")))),
	)

	lc.Append(fx.Hook{
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "observability.NewTraceProvider.OnStop")
			defer span.End()

			if err := tp.Shutdown(ctx); err != nil {
				span.RecordError(err)
				otelzap.L().Ctx(ctx).Error("failed to shutdown trace provider", zap.Error(err))
				return err
			}

			return nil
		},
	})

	return tp
}

func InitTraceProvider(ctx context.Context, tp *tracesdk.TracerProvider) {
	ctx, span := tracer.Start(ctx, "observability.InitTraceProvider")
	defer span.End()

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.TraceContext{})
}
