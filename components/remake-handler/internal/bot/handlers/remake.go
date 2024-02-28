package handlers

import (
	_ "embed"
	"fmt"

	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegoutil"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

func (h *handlers) RemakeCommandHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.RemakeCommandHandler")
	defer span.End()

	if update.Message == nil {
		otelzap.L().Ctx(ctx).Warn("update message is nil")
		return
	}

	area := h.AreasService.RandomArea(ctx)
	msg := telegoutil.Message(
		telegoutil.ID(update.Message.Chat.ID),
		fmt.Sprintf("重开成功，您出生在 <b>%s, %s</b>", area.City, area.Country),
	).
		WithParseMode(telego.ModeHTML).
		WithReplyParameters(&telego.ReplyParameters{MessageID: update.Message.MessageID})

	if _, err := bot.SendMessage(msg); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}

	otelzap.L().Ctx(ctx).Debug("Remake successful", zap.String("city", area.City), zap.String("country", area.Country))
}
