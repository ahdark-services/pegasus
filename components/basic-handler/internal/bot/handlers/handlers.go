package handlers

import (
	"github.com/go-redis/redis_rate/v10"
	"github.com/mymmrac/telego"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/basic-handler/services/action_reply"
	"github.com/ahdark-services/pegasus/components/basic-handler/services/datacenter"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/remake-handler/internal/bot/handlers")

type Handlers interface {
	StartCommandHandler(bot *telego.Bot, update telego.Update)
	ActionCommandHandler(bot *telego.Bot, update telego.Update)
	IDCommandHandler(bot *telego.Bot, update telego.Update)
	DatacenterCommandHandler(bot *telego.Bot, update telego.Update)
	DatacenterMoreInfoHandler(bot *telego.Bot, update telego.Update)
}

type handlers struct {
	fx.In
	ActionReplyService action_reply.Service
	DatacenterService  datacenter.Service
	RedisRateLimiter   *redis_rate.Limiter
}

func NewHandlers(h handlers) Handlers {
	return &h
}
