package utils

import (
	"context"

	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/samber/lo"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

func NewAMQPTableFromCtx(ctx context.Context) amqp.Table {
	mapCarrier := propagation.MapCarrier{}
	propagation.NewCompositeTextMapPropagator(
		propagation.Baggage{},
		propagation.TraceContext{},
	).Inject(ctx, &mapCarrier)

	return lo.MapEntries(mapCarrier, func(k string, v string) (string, interface{}) {
		return k, v
	})
}

func NewCtxFromAMQPTable(ctx context.Context, table amqp.Table) context.Context {
	if table == nil {
		return ctx
	}

	return propagation.NewCompositeTextMapPropagator(propagation.TraceContext{}).
		Extract(ctx, propagation.MapCarrier(lo.MapEntries(table, func(key string, value interface{}) (string, string) {
			return key, value.(string)
		})))
}

func AttributesFromDelivery(delivery amqp.Delivery) []attribute.KeyValue {
	return []attribute.KeyValue{
		attribute.String("amqp.exchange", delivery.Exchange),
		attribute.String("amqp.routing_key", delivery.RoutingKey),
		attribute.String("amqp.consumer_tag", delivery.ConsumerTag),
		attribute.Int64("amqp.delivery_tag", int64(delivery.DeliveryTag)),
		attribute.Bool("amqp.redelivered", delivery.Redelivered),
		attribute.Int64("amqp.message_count", int64(delivery.MessageCount)),
		attribute.String("amqp.content_type", delivery.ContentType),
		attribute.String("amqp.content_encoding", delivery.ContentEncoding),
		attribute.String("amqp.correlation_id", delivery.CorrelationId),
		attribute.String("amqp.reply_to", delivery.ReplyTo),
		attribute.String("amqp.message_id", delivery.MessageId),
		attribute.Int64("amqp.timestamp", delivery.Timestamp.UnixMicro()),
		attribute.String("amqp.type", delivery.Type),
		attribute.String("amqp.user_id", delivery.UserId),
		attribute.String("amqp.app_id", delivery.AppId),
	}
}

func NewAmqpPublishing(ctx context.Context, pushing amqp.Publishing) amqp.Publishing {
	if pushing.Headers == nil {
		pushing.Headers = amqp.Table{}
	}

	if _, ok := pushing.Headers["x-trace"]; !ok {
		pushing.Headers["x-trace"] = NewAMQPTableFromCtx(ctx)
	}

	return pushing
}

func HandleAmqpDelivery(delivery amqp.Delivery, f func(ctx context.Context, delivery amqp.Delivery)) {
	ctx := context.Background()
	if _, ok := delivery.Headers["x-trace"]; ok {
		if _, ok := delivery.Headers["x-trace"].(amqp.Table); ok {
			ctx = NewCtxFromAMQPTable(ctx, delivery.Headers["x-trace"].(amqp.Table))
		}
	}

	ctx, span := tracer.Start(ctx, "HandleAmqpDelivery",
		trace.WithSpanKind(trace.SpanKindConsumer),
		trace.WithAttributes(AttributesFromDelivery(delivery)...),
	)
	defer span.End()

	f(ctx, delivery)
}
