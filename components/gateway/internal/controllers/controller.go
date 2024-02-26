package controllers

import (
	"github.com/ahdark-services/pegasus/components/gateway/internal/controllers/handlers"
	"github.com/cloudwego/hertz/pkg/app/server"
)

func BindRouters(svr *server.Hertz, handlers handlers.Handlers) {
	svr.POST("/update", handlers.UpdateHandler)
}
