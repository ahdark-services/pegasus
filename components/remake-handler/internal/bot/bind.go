package bot

import (
	"github.com/ahdark-services/pegasus/components/remake-handler/internal/bot/handlers"
	"github.com/mymmrac/telego/telegohandler"
)

func BindHandlers(r *telegohandler.BotHandler, handlers handlers.Handlers) {
	r.Handle(handlers.RemakeCommandHandler, telegohandler.CommandEqual("remake"))
}
