package consul

import (
	"context"
	"fmt"

	"github.com/cloudwego/hertz/pkg/app/server/registry"
	consulapi "github.com/hashicorp/consul/api"
	"github.com/hertz-contrib/registry/consul"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/pkg/utils"
)

func NewHertzConsulRegistry(ctx context.Context, serviceName string, client *consulapi.Client, vip *viper.Viper) (registry.Registry, *registry.Info, error) {
	ctx, span := tracer.Start(ctx, "consul.NewHertzConsulRegistry")
	defer span.End()

	localIP, err := utils.GetLocalIP()
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get local ip", zap.Error(err))
		return nil, nil, err
	}

	reg := consul.NewConsulRegister(client, consul.WithCheck(&consulapi.AgentServiceCheck{
		Interval:                       viper.GetString("consul.check.interval"),
		HTTP:                           fmt.Sprintf("http://%s:%d/health", localIP, vip.GetUint16("server.port")),
		Method:                         viper.GetString("consul.check.method"),
		DeregisterCriticalServiceAfter: viper.GetString("consul.check.deregister_critical_service_after"),
	}))

	info := &registry.Info{
		ServiceName: fmt.Sprintf("%s-%s", vip.GetString("name"), serviceName),
		Addr:        utils.NewAddr("tcp", fmt.Sprintf("%s:%d", localIP, vip.GetUint16("server.port"))),
		Weight:      viper.GetInt("consul.weight"),
	}

	return reg, info, nil
}
