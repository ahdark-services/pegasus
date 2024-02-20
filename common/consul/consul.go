package consul

import (
	"context"

	consulapi "github.com/hashicorp/consul/api"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/nextpayt/internal/consul")

func NewConsulConfig(ctx context.Context, vip *viper.Viper) *consulapi.Config {
	ctx, span := tracer.Start(ctx, "consul.NewConsulConfig")
	defer span.End()

	config := consulapi.DefaultConfig()
	config.Address = vip.GetString("consul.address")
	config.Scheme = vip.GetString("consul.scheme")
	config.PathPrefix = vip.GetString("consul.path_prefix")
	config.Datacenter = vip.GetString("consul.datacenter")
	config.Token = vip.GetString("consul.token")
	config.TokenFile = vip.GetString("consul.token_file")
	config.Namespace = vip.GetString("consul.namespace")
	config.Partition = vip.GetString("consul.partition")

	return config
}

// NewConsulClient creates a new consul client.
func NewConsulClient(ctx context.Context, config *consulapi.Config) (*consulapi.Client, error) {
	ctx, span := tracer.Start(ctx, "consul.NewConsulClient")
	defer span.End()

	client, err := consulapi.NewClient(config)
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to create consul client", zap.Error(err))
		span.RecordError(err)
		return nil, err
	}

	return client, nil
}
