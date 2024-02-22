package transport

import (
	"context"
	"github.com/ahdark-services/pegasus/constants"
	"github.com/ahdark-services/pegasus/pkg/utils"
	"github.com/bytedance/sonic"
	"github.com/cloudwego/hertz/pkg/common/bytebufferpool"
	"github.com/mymmrac/telego"
	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/gateway/services/transport")

type Service interface {
	SendUpdate(ctx context.Context, update telego.Update) error
}

type service struct {
	fx.In
	Chan *amqp.Channel
}

func NewService(s service) Service {
	return &s
}

func (s *service) SendUpdate(ctx context.Context, update telego.Update) error {
	ctx, span := tracer.Start(ctx, "TransportService.SendUpdate")
	defer span.End()

	buf := bytebufferpool.Get()
	defer bytebufferpool.Put(buf)

	if err := sonic.ConfigStd.NewEncoder(buf).Encode(update); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to encode update", zap.Error(err))
		return err
	}

	if err := s.Chan.PublishWithContext(
		ctx,
		constants.ExchangeBotUpdates,
		"",
		false,
		false,
		utils.NewAmqpPublishing(ctx, amqp.Publishing{
			Body: buf.Bytes(),
		}),
	); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to publish update", zap.Error(err))
		return err
	}

	return nil
}
