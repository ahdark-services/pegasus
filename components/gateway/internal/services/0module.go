package services

import (
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/gateway/internal/services/transport"
)

func Module() fx.Option {
	return fx.Module("services",
		fx.Provide(transport.NewService),
	)
}
