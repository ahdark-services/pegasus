package observability

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("internal.observability",
		fx.Provide(fx.Annotate(NewResource, fx.ParamTags(``, `name:"serviceName"`))),

		fx.Provide(NewTraceExporter),
		fx.Provide(NewTraceProvider),
		fx.Invoke(InitTraceProvider),

		fx.Provide(NewMeterReader),
		fx.Provide(NewMeterProvider),
		fx.Invoke(InitMeterProvider),
	)
}
