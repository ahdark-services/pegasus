package telegram_bot

import (
	"context"

	"github.com/mymmrac/telego"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/common/tgbot")

func NewBot(ctx context.Context, vip *viper.Viper) (*telego.Bot, error) {
	ctx, span := tracer.Start(ctx, "telegram_bot.NewBot")
	defer span.End()

	bot, err := telego.NewBot(
		vip.GetString("telegram_bot.token"),
		telego.WithLogger(otelzap.L().Sugar().Named("telegram_bot")),
		telego.WithHealthCheck(),
		telego.WithAPIServer(vip.GetString("telegram_bot.api_server")),
	)
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to create telegram bot", zap.Error(err))
		return nil, err
	}

	return bot, nil
}
