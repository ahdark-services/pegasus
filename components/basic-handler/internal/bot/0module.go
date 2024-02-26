package bot

import (
	"github.com/ahdark-services/pegasus/components/basic-handler/internal/bot/handlers"
	"go.uber.org/fx"
)

func Module() fx.Option {
	return fx.Module("handlers",
		fx.Provide(handlers.NewHandlers),

		fx.Invoke(BindHandlers),
	)
}
