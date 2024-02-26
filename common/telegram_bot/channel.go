package telegram_bot

import (
	"context"
	"fmt"
	"github.com/spf13/viper"
	"reflect"

	"github.com/bytedance/sonic"
	"github.com/mymmrac/telego"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/constants"
	"github.com/ahdark-services/pegasus/pkg/utils"
)

func declareExchangeAndQueue(ctx context.Context, serviceName string, ch *amqp.Channel) error {
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
		fmt.Sprintf("bot_updates:queue.%s", serviceName),
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
	vip *viper.Viper,
	conn *amqp.Connection,
	lc fx.Lifecycle,
) (<-chan telego.Update, error) {
	ctx, span := tracer.Start(ctx, "telegram_bot.NewUpdatesChannel")
	defer span.End()

	ch, err := conn.Channel()
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to create channel", zap.Error(err))
		return nil, err
	}

	if err := declareExchangeAndQueue(ctx, serviceName, ch); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to declare exchange and queue", zap.Error(err))
		return nil, err
	}

	msgs, err := ch.Consume(
		fmt.Sprintf("bot_updates:queue.%s", serviceName),
		vip.GetString("instance_id"),
		true,
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
			if err := recover(); err != nil {
				otelzap.L().Panic("telegram bot update channel closed", zap.Any("error", err))
			}
		}()

		for msg := range msgs {
			utils.HandleAmqpDelivery(msg, func(ctx context.Context, delivery amqp.Delivery) {
				var u telego.Update
				if err := sonic.Unmarshal(delivery.Body, &u); err != nil {
					otelzap.L().Warn("failed to unmarshal message", zap.Error(err))
					return
				}

				u = u.WithContext(ctx)

				update <- u
			})
		}
	}()

	lc.Append(fx.Hook{
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "telegram_bot.StopWebhook")
			defer span.End()

			if err := ch.Cancel(vip.GetString("instance_id"), false); err != nil {
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
