package server

import (
	"context"
	"fmt"

	hertzloggerzap "github.com/ahdark-services/pegasus/pkg/hertzloggerzap"
	hertzprometheus "github.com/ahdark-services/pegasus/pkg/hertzprometheus"
	"github.com/cloudwego/hertz/pkg/app/server"
	hertzregistry "github.com/cloudwego/hertz/pkg/app/server/registry"
	"github.com/cloudwego/hertz/pkg/common/hlog"
	"github.com/cloudwego/hertz/pkg/network/netpoll"
	hertztracing "github.com/hertz-contrib/obs-opentelemetry/tracing"
	promclient "github.com/prometheus/client_golang/prometheus"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/fx"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/commin/server")

func NewServer(
	ctx context.Context,
	vip *viper.Viper,
	lc fx.Lifecycle,
	promRegistry *promclient.Registry,
	serviceRegistry hertzregistry.Registry,
	serviceInfo *hertzregistry.Info,
) (*server.Hertz, error) {
	ctx, span := tracer.Start(ctx, "server.NewServer")
	defer span.End()

	hlog.SetLogger(hertzloggerzap.NewLoggerWithZapLogger(otelzap.L().Named("hertz")))

	traceOption, cfg := hertztracing.NewServerTracer()
	svr := server.Default(
		traceOption,
		server.WithRegistry(serviceRegistry, serviceInfo),
		server.WithNetwork(vip.GetString("server.network")),
		server.WithHostPorts(fmt.Sprintf("%s:%d", vip.GetString("server.address"), vip.GetInt("server.port"))),
		server.WithHandleMethodNotAllowed(true),
		server.WithTracer(hertzprometheus.NewServerTracer(
			"",
			"",
			hertzprometheus.WithRegistry(promRegistry),
			hertzprometheus.WithEnableGoCollector(false),
			hertzprometheus.WithDisableServer(true),
		)),
		server.WithTransport(netpoll.NewTransporter),
	)
	svr.Use(hertztracing.ServerMiddleware(cfg))

	svr.GET("/health", HealthHandler)
	svr.NoRoute(NoRouteHandler)
	svr.NoMethod(NoMethodHandler)

	lc.Append(fx.Hook{
		OnStart: func(_ context.Context) error {
			go svr.Spin()

			return nil
		},
	})

	return svr, nil
}
