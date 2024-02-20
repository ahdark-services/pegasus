package dcron

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("internal.dcron",
		fx.Provide(newLogger),
		fx.Provide(fx.Annotate(NewDCron, fx.ParamTags(``, `name:"serviceName"`))),
	)
}
