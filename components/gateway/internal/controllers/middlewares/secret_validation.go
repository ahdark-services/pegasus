package middlewares

import (
	"context"

	"github.com/cloudwego/hertz/pkg/app"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
)

func TelegramWebhookSecretValidation(vip *viper.Viper) app.HandlerFunc {
	if vip.GetString("telegram_bot.webhook.secret_token") == "" {
		return func(ctx context.Context, c *app.RequestContext) {
			otelzap.L().Ctx(ctx).Debug("Telegram webhook secret is not set")

			c.Next(ctx)
		}
	}

	return func(ctx context.Context, c *app.RequestContext) {
		ctx, span := tracer.Start(ctx, "middlewares.TelegramWebhookSecretValidation")
		defer span.End()

		secret := c.Request.Header.Get("X-Telegram-Webhook-Secret")
		if secret != vip.GetString("telegram_bot.webhook.secret_token") {
			c.AbortWithStatus(403)
			otelzap.L().Ctx(ctx).Warn("Telegram webhook secret is invalid")
			return
		}

		otelzap.L().Ctx(ctx).Debug("Telegram webhook secret is valid")
		c.Next(ctx)
	}
}
