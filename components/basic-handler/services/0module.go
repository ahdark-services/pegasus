package services

import (
	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/basic-handler/services/action_reply"
	"github.com/ahdark-services/pegasus/components/basic-handler/services/datacenter"
)

func Module() fx.Option {
	return fx.Module("services",
		fx.Provide(action_reply.NewService),
		fx.Provide(datacenter.NewService),
	)
}
