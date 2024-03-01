package handlers

import (
	_ "embed"
	"html/template"

	"github.com/cloudwego/hertz/pkg/common/bytebufferpool"
	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegoutil"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

//go:embed id.tpl
var idTemplateText string
var idTemplate = template.Must(template.New("id").Parse(idTemplateText))

func (h *handlers) IDCommandHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.IDCommandHandler")
	defer span.End()

	if update.Message == nil {
		otelzap.L().Ctx(ctx).Warn("update message is nil")
		return
	}

	buf := bytebufferpool.Get()
	defer bytebufferpool.Put(buf)

	if err := idTemplate.Execute(buf, update); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to execute template", zap.Error(err))
		return
	}

	msg := telegoutil.Message(telegoutil.ID(update.Message.Chat.ID), buf.String()).
		WithParseMode(telego.ModeHTML).
		WithReplyParameters(&telego.ReplyParameters{MessageID: update.Message.MessageID})

	if _, err := bot.SendMessage(msg); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}
}
