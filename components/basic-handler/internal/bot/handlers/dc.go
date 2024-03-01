package handlers

import (
	_ "embed"
	"fmt"
	"html/template"

	"github.com/cloudwego/hertz/pkg/common/bytebufferpool"
	"github.com/go-redis/redis_rate/v10"
	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegoutil"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

//go:embed dc.tpl
var dcTemplateText string

func (h *handlers) DatacenterCommandHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.DatacenterCommandHandler")
	defer span.End()

	if update.Message == nil {
		otelzap.L().Ctx(ctx).Warn("update message is nil")
		return
	}

	if res, err := h.RedisRateLimiter.Allow(ctx, fmt.Sprintf("rate:basic-handler:handler:datacenter_command_handler:chat-%d", update.Message.Chat.ID), redis_rate.PerSecond(1)); err != nil {
		otelzap.L().Ctx(ctx).Error("rate limit exceeded", zap.Error(err))
		_, _ = bot.SendMessage(telegoutil.Message(telegoutil.ID(update.Message.Chat.ID), "rate limit exceeded").WithReplyParameters(&telego.ReplyParameters{MessageID: update.Message.MessageID}))
		return
	} else if res.Allowed == 0 {
		otelzap.L().Ctx(ctx).Warn("rate limit exceeded")
		return
	}

	var funcMap = template.FuncMap{
		"datacenter": func(username string) int {
			dc, err := h.DatacenterService.QueryDatacenterByUsername(ctx, username)
			if err != nil {
				otelzap.L().Ctx(ctx).Error("failed to query datacenter", zap.Error(err))
				return 0
			}

			return dc
		},
	}

	tpl, err := template.New("datacenter").Funcs(funcMap).Parse(dcTemplateText)
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to parse template", zap.Error(err))
		return
	}

	buf := bytebufferpool.Get()
	defer bytebufferpool.Put(buf)

	if err := tpl.Execute(buf, update); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to execute template", zap.Error(err))
		return
	}

	if _, err := bot.SendMessage(
		telegoutil.
			Message(telegoutil.ID(update.Message.Chat.ID), buf.String()).
			WithParseMode(telego.ModeHTML).
			WithReplyParameters(&telego.ReplyParameters{MessageID: update.Message.MessageID}).
			WithReplyMarkup(telegoutil.InlineKeyboard(
				telegoutil.InlineKeyboardRow(
					telegoutil.InlineKeyboardButton("更多信息").WithCallbackData("datacenter_more_info"),
				),
			)),
	); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}
}

func (h *handlers) DatacenterMoreInfoHandler(bot *telego.Bot, update telego.Update) {
	ctx, span := tracer.Start(update.Context(), "handlers.DatacenterMoreInfoHandler")
	defer span.End()

	if !update.CallbackQuery.Message.IsAccessible() {
		otelzap.L().Ctx(ctx).Warn("message is not accessible")
		return
	}

	chat := update.CallbackQuery.Message.GetChat()
	if res, err := h.RedisRateLimiter.Allow(ctx, fmt.Sprintf("rate:basic-handler:handler:datacenter_more_info:chat-%d", chat.ID), redis_rate.PerSecond(1)); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to rate limit", zap.Error(err))
		return
	} else if res.Allowed == 0 {
		otelzap.L().Ctx(ctx).Warn("rate limit exceeded")
		return
	}

	if _, err := bot.SendMessage(
		telegoutil.
			Message(chat.ChatID(), `DC1: 美国 迈阿密
DC2: 荷兰 阿姆斯特丹
DC3: 美国 迈阿密
DC4: 荷兰 阿姆斯特丹
DC5: 新加坡

<a href="https://t.me/KinhRoBotChannel/88">注册手机区号对应数据中心信息</a>`).
			WithParseMode(telego.ModeHTML),
	); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send message", zap.Error(err))
		return
	}
}
