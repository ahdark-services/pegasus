package observability

import (
	"context"
	"fmt"

	"github.com/spf13/viper"
	"go.opentelemetry.io/otel/sdk/resource"
	semconv "go.opentelemetry.io/otel/semconv/v1.24.0"
)

func NewResource(ctx context.Context, serviceName string, vip *viper.Viper) (*resource.Resource, error) {
	ctx, span := tracer.Start(ctx, "observability.NewResource")
	defer span.End()

	return resource.New(ctx,
		resource.WithFromEnv(),
		resource.WithTelemetrySDK(),
		resource.WithContainer(),

		resource.WithSchemaURL(semconv.SchemaURL),
		resource.WithAttributes(
			semconv.ServiceNamespaceKey.String(vip.GetString("namespace")),
			semconv.ServiceNameKey.String(fmt.Sprintf("%s-%s", vip.GetString("name"), serviceName)),
			semconv.ServiceVersionKey.String(vip.GetString("version")),
			semconv.ServiceInstanceIDKey.String(vip.GetString("instance_id")),
		),
	)
}
