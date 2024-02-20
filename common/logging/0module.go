package logging

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("internal.logging",
		fx.Provide(
			fx.Annotate(callerOption, fx.ResultTags(`group:"logging_options"`)),
			fx.Annotate(stacktraceOption, fx.ResultTags(`group:"logging_options"`)),
			fx.Annotate(coreOption, fx.ResultTags(`group:"logging_options"`)),
		),

		fx.Provide(fx.Annotate(NewLogger, fx.ParamTags(``, ``, `group:"logging_options"`))),
		fx.Invoke(UseLogger),
	)
}
