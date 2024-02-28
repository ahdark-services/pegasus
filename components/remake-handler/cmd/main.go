package main

import (
	"context"

	"go.uber.org/fx"

	"github.com/ahdark-services/pegasus/components/remake-handler/internal/bot"
	"github.com/ahdark-services/pegasus/components/remake-handler/internal/services"
	"github.com/ahdark-services/pegasus/entry"
)

func main() {
	app := fx.New(
		fx.Supply(fx.Annotate(context.Background(), fx.As(new(context.Context)))),
		fx.Supply(fx.Annotate("remake-handler", fx.ResultTags(`name:"serviceName"`))),

		entry.AppEntries(),

		services.Module(),
		bot.Module(),
	)

	app.Run()
}
