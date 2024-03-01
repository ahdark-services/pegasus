package handlers

import (
	_ "embed"

	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegoutil"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel/attribute"
	"go.uber.org/zap"
)

func (h *handlers) ActionCommandHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.ActionCommandHandler")
	defer span.End()

	if update.Message == nil {
		otelzap.L().Ctx(ctx).Warn("update message is nil")
		return
	}

	if !h.ActionReplyService.CheckNeedReply(ctx, update.Message.Text) {
		span.SetAttributes(attribute.Bool("need_reply", false))
		otelzap.L().Ctx(ctx).Debug("update message is not need reply")
		return
	}

	if update.Message.ReplyToMessage == nil {
		otelzap.L().Ctx(ctx).Warn("update message reply to message is nil")
		_, _ = bot.SendMessage(
			telegoutil.Message(telegoutil.ID(update.Message.Chat.ID), "请回复一条消息").
				WithReplyParameters(&telego.ReplyParameters{MessageID: update.Message.MessageID}),
		)
		return
	}

	tpl, err := h.ActionReplyService.GetReplyTemplate(
		ctx,
		update.Message.Text,
		update.Message.From,
		update.Message.ReplyToMessage.From,
	)
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to get reply template", zap.Error(err))
		return
	}

	msg := telegoutil.Message(telegoutil.ID(update.Message.Chat.ID), tpl).
		WithParseMode(telego.ModeHTML)

	if _, err := bot.SendMessage(msg); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}
}
