package telegram_bot

import (
	"context"

	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegohandler"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

func NewHandler(bot *telego.Bot, ch <-chan telego.Update, lc fx.Lifecycle) (*telegohandler.BotHandler, error) {
	h, err := telegohandler.NewBotHandler(bot, ch)
	if err != nil {
		otelzap.L().Error("failed to create telegram bot handler", zap.Error(err))
		return nil, err
	}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StartHandler")
			defer span.End()

			go h.Start()

			return nil
		},
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StopHandler")
			defer span.End()

			h.Stop()

			return nil
		},
	})

	return h, nil
}
