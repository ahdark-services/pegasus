package settings

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module(
		"settings",
		fx.Provide(NewSettings),
	)
}
