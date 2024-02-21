package telegram_bot

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module("telegram_bot",
		fx.Provide(NewBot),
		fx.Provide(fx.Annotate(NewWebhookChannel, fx.ParamTags(``, `name:"serviceName"`))),
		fx.Provide(NewHandler),
	)
}
