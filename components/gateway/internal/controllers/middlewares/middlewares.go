package middlewares

import "go.opentelemetry.io/otel"

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/gateway/internal/controllers/middlewares")
