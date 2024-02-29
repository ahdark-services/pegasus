package main

import (
	"context"
	"github.com/ahdark-services/pegasus/components/any-action-handler/services"

	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/any-action-handler/internal/bot"
	"github.com/ahdark-services/pegasus/entry"
)

func main() {
	app := fx.New(
		fx.Supply(fx.Annotate(context.Background(), fx.As(new(context.Context)))),
		fx.Supply(fx.Annotate("any-action-handler", fx.ResultTags(`name:"serviceName"`))),

		entry.AppEntries(),

		services.Module(),
		bot.Module(),
	)

	app.Run()
}
