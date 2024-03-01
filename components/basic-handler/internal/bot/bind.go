package bot

import (
	"github.com/ahdark-services/pegasus/components/basic-handler/internal/bot/handlers"
	"github.com/ahdark-services/pegasus/pkg/utils"
	"github.com/mymmrac/telego/telegohandler"
)

func BindHandlers(r *telegohandler.BotHandler, handlers handlers.Handlers) {
	// start command
	r.Handle(handlers.StartCommandHandler, telegohandler.CommandEqual("start"), utils.PrivateChatOnly())

	// id command
	r.Handle(handlers.IDCommandHandler, telegohandler.CommandEqual("id"))

	// action command
	r.Handle(handlers.ActionCommandHandler, telegohandler.AnyMessageWithText(), telegohandler.TextPrefix("/"), telegohandler.Not(utils.PrivateChatOnly()))
}
