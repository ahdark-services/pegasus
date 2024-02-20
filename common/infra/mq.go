package infra

import (
	"context"
	"fmt"

	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

func NewAMQPConn(ctx context.Context, vip *viper.Viper) (*amqp.Connection, error) {
	ctx, span := tracer.Start(ctx, "mq.NewAMQPConn")
	defer span.End()

	url := fmt.Sprintf("amqp://%s:%s@%s:%d/",
		vip.GetString("mq.username"),
		vip.GetString("mq.password"),
		vip.GetString("mq.host"),
		vip.GetUint16("mq.port"),
	)

	conn, err := amqp.DialConfig(url, amqp.Config{
		Heartbeat: 10,
		Vhost:     vip.GetString("mq.vhost"),
	})
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to connect to rabbitmq", zap.Error(err))
		return nil, err
	}

	return conn, nil
}

func NewAMQPChannel(ctx context.Context, conn *amqp.Connection, lc fx.Lifecycle) (*amqp.Channel, error) {
	ctx, span := tracer.Start(ctx, "mq.NewAMQPChannel")
	defer span.End()

	ch, err := conn.Channel()
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to create a channel", zap.Error(err))
		return nil, err
	}

	c := make(chan *amqp.Error)
	ch.NotifyClose(c)

	go func() {
		for err := range c {
			otelzap.L().Ctx(ctx).Panic("mq channel closed", zap.Error(err))
			return
		}
	}()

	lc.Append(fx.Hook{
		OnStop: func(ctx context.Context) error {
			ctx, span := tracer.Start(ctx, "mq.NewAMQPChannel.OnStop")
			defer span.End()

			if err := ch.Close(); err != nil {
				span.RecordError(err)
				otelzap.L().Ctx(ctx).Error("failed to close channel", zap.Error(err))
				return err
			}

			return nil
		},
	})

	return ch, nil
}
