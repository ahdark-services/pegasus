package consul

import (
	"context"
	"fmt"

	"github.com/cloudwego/kitex/pkg/discovery"
	kitexregistry "github.com/cloudwego/kitex/pkg/registry"
	consulapi "github.com/hashicorp/consul/api"
	consul "github.com/kitex-contrib/registry-consul"
	kitexconsul "github.com/kitex-contrib/registry-consul"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/pkg/utils"
)

func NewKitexConsulRegistry(ctx context.Context, config *consulapi.Config, vip *viper.Viper) (kitexregistry.Registry, error) {
	ctx, span := tracer.Start(ctx, "consul.NewKitexConsulRegistry")
	defer span.End()

	localIP, err := utils.GetLocalIP()
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to get local ip", zap.Error(err))
		return nil, err
	}

	r, err := kitexconsul.NewConsulRegisterWithConfig(config, kitexconsul.WithCheck(&consulapi.AgentServiceCheck{
		Interval:                       vip.GetString("consul.check.interval"),
		GRPC:                           fmt.Sprintf("%s:%d", localIP, vip.GetUint16("rpc.port")),
		GRPCUseTLS:                     false,
		DeregisterCriticalServiceAfter: vip.GetString("consul.check.deregister_critical_service_after"),
	}))
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to create kitex consul register", zap.Error(err))
		return nil, err
	}

	return r, nil
}

func NewKitexConsulResolver(ctx context.Context, config *consulapi.Config) (discovery.Resolver, error) {
	ctx, span := tracer.Start(ctx, "consul.NewKitexConsulResolver")
	defer span.End()

	resolver, err := consul.NewConsulResolverWithConfig(config)
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to create kitex consul resolver", zap.Error(err))
		span.RecordError(err)
		return nil, err
	}

	return resolver, nil
}
