package handlers

import (
	"context"
	"github.com/cloudwego/hertz/pkg/app"
	amqp "github.com/rabbitmq/amqp091-go"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/gateway/internal/services/transport"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/gateway/internal/handlers")

type Handlers interface {
	UpdateHandler(ctx context.Context, c *app.RequestContext)
}

type handlers struct {
	fx.In
	AMQPChannel      *amqp.Channel
	TransportService transport.Service
}

func NewHandlers(h handlers) Handlers {
	return &h
}
