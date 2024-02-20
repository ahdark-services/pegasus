package infra

import (
	"context"

	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/namespace"
	"go.uber.org/zap"

	"github.com/ahdark-services/pegasus/pkg/utils"
)

func NewEtcdClient(ctx context.Context, vip *viper.Viper, logger *otelzap.Logger) (*clientv3.Client, error) {
	ctx, span := tracer.Start(ctx, "infra.NewEtcdClient")
	defer span.End()

	tlsConfig, err := utils.NewTLSConfig(ctx, utils.TLSParams{
		CACertPath:         vip.GetString("etcd.tls.ca_file"),
		ClientCertPath:     vip.GetString("etcd.tls.cert_file"),
		ClientKeyPath:      vip.GetString("etcd.tls.key_file"),
		InsecureSkipVerify: vip.GetBool("etcd.tls.insecure"),
	})
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to create tls config", zap.Error(err))
		return nil, err
	}

	client, err := clientv3.New(clientv3.Config{
		Endpoints: vip.GetStringSlice("etcd.endpoints"),
		Username:  vip.GetString("etcd.username"),
		Password:  vip.GetString("etcd.password"),
		Logger:    logger.Named("etcd"),
		TLS:       tlsConfig,
	})
	if err != nil {
		span.RecordError(err)
		otelzap.L().Ctx(ctx).Error("failed to create etcd client", zap.Error(err))
		return nil, err
	}

	client.KV = namespace.NewKV(client.KV, "cecilia-card-backend:")
	client.Watcher = namespace.NewWatcher(client.Watcher, "cecilia-card-backend:")
	client.Lease = namespace.NewLease(client.Lease, "cecilia-card-backend:")

	return client, nil
}
