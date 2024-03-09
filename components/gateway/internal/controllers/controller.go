package controllers

import (
	"github.com/cloudwego/hertz/pkg/app/server"
	"github.com/spf13/viper"

	"github.com/ahdark-services/pegasus/components/gateway/internal/controllers/handlers"
	"github.com/ahdark-services/pegasus/components/gateway/internal/controllers/middlewares"
)

func BindRouters(svr *server.Hertz, handlers handlers.Handlers, vip *viper.Viper) {
	svr.POST("/update", middlewares.TelegramWebhookSecretValidation(vip), handlers.UpdateHandler)
}
