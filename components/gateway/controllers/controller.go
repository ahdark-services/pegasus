package controllers

import (
	"github.com/cloudwego/hertz/pkg/app/server"
	"go.opentelemetry.io/otel"

	"github.com/ahdark-services/pegasus/components/gateway/controllers/handlers"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/gateway/controllers")

func BindRouters(svr *server.Hertz, handlers handlers.Handlers) {
	svr.POST("/update", handlers.UpdateHandler)
}
