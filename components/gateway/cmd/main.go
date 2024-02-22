package main

import (
	"context"

	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/gateway/bot"
	"github.com/ahdark-services/pegasus/components/gateway/controllers"
	"github.com/ahdark-services/pegasus/components/gateway/services"
	"github.com/ahdark-services/pegasus/entry"
)

func main() {
	app := fx.New(
		fx.Supply(fx.Annotate(context.Background(), fx.As(new(context.Context)))),
		fx.Supply(fx.Annotate("gateway", fx.ResultTags(`name:"serviceName"`))),

		entry.AppEntries(),

		controllers.Module(),
		services.Module(),
		bot.Module(),
	)

	app.Run()
}
