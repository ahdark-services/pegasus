package consul

import "go.uber.org/fx"

func Module() fx.Option {
	return fx.Module(
		"internal.consul",
		fx.Provide(NewConsulConfig),
		fx.Provide(NewConsulClient),
		fx.Provide(NewKitexConsulRegistry),
		fx.Provide(NewKitexConsulResolver),
		fx.Provide(fx.Annotate(NewHertzConsulRegistry, fx.ParamTags(``, `name:"serviceName"`))),
	)
}
