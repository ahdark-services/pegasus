package handlers

import (
	"github.com/mymmrac/telego"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/remake-handler/internal/bot/handlers")

type Handlers interface {
	RemakeCommandHandler(bot *telego.Bot, update telego.Update)
}

type handlers struct {
	fx.In
}

func NewHandlers(h handlers) Handlers {
	return &h
}
