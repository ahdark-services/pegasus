package telegram_bot

import (
	"context"

	"github.com/cloudwego/hertz/pkg/app"
	"github.com/cloudwego/hertz/pkg/app/server"
	"github.com/mymmrac/telego"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/common/serializer"
)

func newWebhookParams(vip *viper.Viper) *telego.SetWebhookParams {
	return &telego.SetWebhookParams{
		URL:                vip.GetString("telegram_bot.webhook.url"),
		IPAddress:          vip.GetString("telegram_bot.webhook.ip_address"),
		MaxConnections:     vip.GetInt("telegram_bot.webhook.max_connections"),
		AllowedUpdates:     vip.GetStringSlice("telegram_bot.webhook.allowed_updates"),
		DropPendingUpdates: vip.GetBool("telegram_bot.webhook.drop_pending_updates"),
		SecretToken:        vip.GetString("telegram_bot.webhook.secret_token"),
	}
}

func NewWebhookChannel(bot *telego.Bot, svr *server.Hertz, webhookParams *telego.SetWebhookParams, lc fx.Lifecycle) (<-chan telego.Update, error) {
	if err := bot.SetWebhook(webhookParams); err != nil {
		otelzap.L().Panic("failed to set webhook", zap.Error(err))
		return nil, err
	}

	update, err := bot.UpdatesViaWebhook("/api/v1/telegram/webhook", telego.WithWebhookServer(telego.FuncWebhookServer{
		Server: telego.NoOpWebhookServer{},
		RegisterHandlerFunc: func(path string, handler telego.WebhookHandler) error {
			svr.POST(path, func(ctx context.Context, c *app.RequestContext) {
				ctx, span := tracer.Start(ctx, "telegram_bot.WebhookHandler")
				defer span.End()

				body, err := c.Body()
				if err != nil {
					span.RecordError(err)
					otelzap.L().Ctx(ctx).Error("failed to read request body", zap.Error(err))
					serializer.NewAppResponseError(serializer.CodeErrInvalidParameter, err).AbortWithStatusJSON(c, 400)
					return
				}

				if err := handler(ctx, body); err != nil {
					span.RecordError(err)
					otelzap.L().Ctx(ctx).Error("failed to handle webhook", zap.Error(err))
					serializer.NewAppResponseError(serializer.CodeErrServiceError, err).AbortWithStatusJSON(c, 500)
					return
				}

				serializer.NewAppResponseSuccess(nil).JSON(c)
			})

			return nil
		},
	}))
	if err != nil {
		otelzap.L().Panic("failed to set webhook", zap.Error(err))
		return nil, err
	}

	lc.Append(fx.Hook{
		OnStart: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StartWebhook")
			defer span.End()

			if err := bot.StartWebhook(""); err != nil {
				span.RecordError(err)
				otelzap.L().Ctx(ctx).Error("failed to start webhook", zap.Error(err))
				return err
			}

			return nil
		},
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StopWebhook")
			defer span.End()

			if err := bot.StopWebhookWithContext(ctx); err != nil {
				span.RecordError(err)
				otelzap.L().Ctx(ctx).Error("failed to stop webhook", zap.Error(err))
				return err
			}

			return nil
		},
	})

	return update, nil
}
