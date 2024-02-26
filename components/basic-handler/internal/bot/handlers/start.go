package handlers

import (
	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegoutil"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

func (h *handlers) StartCommandHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.StartCommandHandler")
	defer span.End()

	if _, err := bot.SendMessage(telegoutil.Message(telegoutil.ID(update.Message.Chat.ID), "Hello!")); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}
}
