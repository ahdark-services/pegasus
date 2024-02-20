package utils

import "go.opentelemetry.io/otel"

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/pkg/utils")
