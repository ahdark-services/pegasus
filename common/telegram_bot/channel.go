package telegram_bot

import (
	"context"
	"fmt"
	"reflect"

	"github.com/bytedance/sonic"
	"github.com/mymmrac/telego"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/constants"
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

func declareExchangeAndQueue(ctx context.Context, ch *amqp.Channel) error {
	ctx, span := tracer.Start(ctx, "telegram_bot.declareExchangeAndQueue")
	defer span.End()

	if err := ch.ExchangeDeclare(
		constants.ExchangeBotUpdates,
		amqp.ExchangeFanout,
		false,
		true,
		false,
		false,
		nil,
	); err != nil {
		otelzap.L().Ctx(ctx).Panic("failed to declare exchange", zap.Error(err))
		return err
	}

	q, err := ch.QueueDeclare(
		constants.QueueBotUpdates,
		false,
		true,
		false,
		false,
		nil,
	)
	if err != nil {
		otelzap.L().Ctx(ctx).Panic("failed to declare queue", zap.Error(err))
		return err
	}

	if err := ch.QueueBind(
		q.Name,
		"",
		constants.ExchangeBotUpdates,
		false,
		nil,
	); err != nil {
		otelzap.L().Ctx(ctx).Panic("failed to bind queue", zap.Error(err))
		return err
	}

	return nil
}

func init() {
	if err := sonic.Pretouch(reflect.TypeOf(telego.Update{})); err != nil {
		panic(err)
	}
}

func NewUpdatesChannel(
	ctx context.Context,
	serviceName string,
	conn *amqp.Connection,
	vip *viper.Viper,
	lc fx.Lifecycle,
) (<-chan telego.Update, error) {
	ctx, span := tracer.Start(ctx, "telegram_bot.NewUpdatesChannel")
	defer span.End()

	ch, err := conn.Channel()
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to create channel", zap.Error(err))
		return nil, err
	}

	if err := declareExchangeAndQueue(ctx, ch); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to declare exchange and queue", zap.Error(err))
		return nil, err
	}

	consumer := fmt.Sprintf("%s:%s:%s", constants.QueueBotUpdates, serviceName, vip.GetString("instance_id"))
	msg, err := ch.Consume(
		constants.QueueBotUpdates,
		consumer,
		false,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to consume message", zap.Error(err))
		return nil, err
	}

	update := make(chan telego.Update)
	go func() {
		defer func() {
			otelzap.L().Panic("telegram bot update channel closed")
		}()

		for m := range msg {
			var u telego.Update
			if err := sonic.Unmarshal(m.Body, &u); err != nil {
				otelzap.L().Warn("failed to unmarshal message", zap.Error(err))
				continue
			}

			update <- u
		}
	}()

	lc.Append(fx.Hook{
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StopWebhook")
			defer span.End()

			if err := ch.Cancel(consumer, false); err != nil {
				otelzap.L().Ctx(ctx).Error("failed to cancel consumer", zap.Error(err))
				return err
			}

			if err := ch.Close(); err != nil {
				otelzap.L().Ctx(ctx).Error("failed to close channel", zap.Error(err))
				return err
			}

			return nil
		},
	})

	return update, nil
}
