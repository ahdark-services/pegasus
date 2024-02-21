package main

import (
	"context"
	"github.com/ahdark-services/pegasus/entry"
	"go.uber.org/fx"
)

func main() {
	app := fx.New(
		fx.Supply(fx.Annotate(context.Background(), fx.As(new(context.Context)))),

		entry.AppEntries(),
	)

	app.Run()
}
