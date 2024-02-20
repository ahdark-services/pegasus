package server

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("server",
		fx.Provide(NewServer),
	)
}
