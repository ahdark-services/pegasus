package bot

import (
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/any-action-handler/internal/bot/handlers"
)

func Module() fx.Option {
	return fx.Module("handlers",
		fx.Provide(handlers.NewHandlers),

		fx.Invoke(BindHandlers),
	)
}
