package handlers

import (
	"github.com/mymmrac/telego"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/remake-handler/internal/services/areas"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/basic-handler/internal/bot/handlers")

type Handlers interface {
	RemakeCommandHandler(bot *telego.Bot, update telego.Update)
}

type handlers struct {
	fx.In
	AreasService areas.Service
}

func NewHandlers(h handlers) Handlers {
	return &h
}
