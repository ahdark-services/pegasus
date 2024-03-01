package handlers

import (
	"github.com/mymmrac/telego"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/basic-handler/services/action_reply"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/remake-handler/internal/bot/handlers")

type Handlers interface {
	StartCommandHandler(bot *telego.Bot, update telego.Update)
	ActionCommandHandler(bot *telego.Bot, update telego.Update)
	IDCommandHandler(bot *telego.Bot, update telego.Update)
}

type handlers struct {
	fx.In
	ActionReplyService action_reply.Service
}

func NewHandlers(h handlers) Handlers {
	return &h
}
