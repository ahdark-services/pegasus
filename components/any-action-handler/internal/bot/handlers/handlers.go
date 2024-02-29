package handlers

import (
	"github.com/mymmrac/telego"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/any-action-handler/services/action_reply"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/any-action-handler/internal/bot/handlers")

type Handlers interface {
	ActionCommandHandler(bot *telego.Bot, update telego.Update)
}

type handlers struct {
	fx.In
	ActionReplyService action_reply.Service
}

func NewHandlers(h handlers) Handlers {
	return &h
}
