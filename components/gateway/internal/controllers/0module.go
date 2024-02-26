package controllers

import (
	"github.com/ahdark-services/pegasus/components/gateway/internal/controllers/handlers"
	"go.uber.org/fx"
)

func Module() fx.Option {
	return fx.Module("controllers",
		fx.Provide(handlers.NewHandlers),

		fx.Invoke(BindRouters),
	)
}
