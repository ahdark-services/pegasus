package telegram_bot

import (
	"context"
	"fmt"
	"reflect"

	"github.com/bytedance/sonic"
	"github.com/cloudwego/hertz/pkg/app"
	"github.com/cloudwego/hertz/pkg/app/server"
	"github.com/mymmrac/telego"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/common/serializer"
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

func NewWebhookChannel(
	ctx context.Context,
	serviceName string,
	bot *telego.Bot,
	conn *amqp.Connection,
	svr *server.Hertz,
	vip *viper.Viper,
	lc fx.Lifecycle,
) (<-chan telego.Update, error) {
	ctx, span := tracer.Start(ctx, "telegram_bot.NewWebhookChannel")
	defer span.End()

	switch serviceName {
	case "gateway":
		webhookParams := newWebhookParams(vip)

		if err := bot.SetWebhook(webhookParams); err != nil {
			otelzap.L().Ctx(ctx).Panic("failed to set webhook", zap.Error(err))
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
			otelzap.L().Ctx(ctx).Panic("failed to set webhook", zap.Error(err))
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
	default:
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
}
