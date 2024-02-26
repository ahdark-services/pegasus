package main

import (
	"context"

	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/basic-handler/internal/bot"
	"github.com/ahdark-services/pegasus/entry"
)

func main() {
	app := fx.New(
		fx.Supply(fx.Annotate(context.Background(), fx.As(new(context.Context)))),
		fx.Supply(fx.Annotate("basic-handler", fx.ResultTags(`name:"serviceName"`))),

		entry.AppEntries(),

		bot.Module(),
	)

	app.Run()
}
