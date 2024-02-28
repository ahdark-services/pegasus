package services

import (
	"github.com/ahdark-services/pegasus/components/remake-handler/internal/services/areas"
	"go.uber.org/fx"
)

func Module() fx.Option {
	return fx.Module("services",
		fx.Provide(areas.NewService),
	)
}
