package main

import (
	"context"

	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/gateway/internal/bot"
	"github.com/ahdark-services/pegasus/components/gateway/internal/controllers"
	"github.com/ahdark-services/pegasus/components/gateway/internal/services"
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
