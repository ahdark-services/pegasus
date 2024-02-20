package config

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("internal.config",
		fx.Provide(NewViper),
	)
}
