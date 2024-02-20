package dcron

import (
	"context"

	"github.com/libi/dcron"
	"github.com/libi/dcron/dlog"
	"github.com/libi/dcron/driver"
	clientv3 "go.etcd.io/etcd/client/v3"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/common/dcron")

func NewDCron(ctx context.Context, serviceName string, etcdClient *clientv3.Client, logger dlog.Logger, lc fx.Lifecycle) *dcron.Dcron {
	ctx, span := tracer.Start(ctx, "dcron.NewDCron")
	defer span.End()

	d := dcron.NewDcronWithOption(serviceName,
		driver.NewEtcdDriver(etcdClient),
		dcron.WithLogger(logger),
	)

	lc.Append(fx.Hook{
		OnStart: func(context.Context) error {
			d.Start()
			return nil
		},
		OnStop: func(context.Context) error {
			d.Stop()
			return nil
		},
	})

	return d
}
