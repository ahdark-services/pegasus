package services

import (
	"github.com/ahdark-services/pegasus/components/any-action-handler/services/action_reply"
	"go.uber.org/fx"
)

func Module() fx.Option {
	return fx.Module("services",
		fx.Provide(action_reply.NewService),
	)
}
