package middlewares

import (
	"context"
	"github.com/cloudwego/hertz/pkg/app"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
)

const TelegramBotApiSecretTokenHeader = "X-Telegram-Bot-Api-Secret-Token"

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

		secret := c.Request.Header.Get(TelegramBotApiSecretTokenHeader)
		span.SetAttributes(attribute.String("http.header.secret_token", secret))
		if secret != vip.GetString("telegram_bot.webhook.secret_token") {
			span.SetStatus(codes.Error, "invalid secret token")
			otelzap.L().Ctx(ctx).Warn("Telegram webhook secret is invalid")
			c.AbortWithStatus(403)
			return
		}

		otelzap.L().Ctx(ctx).Debug("Telegram webhook secret is valid")
		c.Next(ctx)
	}
}
