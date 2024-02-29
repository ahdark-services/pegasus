package bot

import (
	"github.com/mymmrac/telego/telegohandler"

	"github.com/ahdark-services/pegasus/components/any-action-handler/internal/bot/handlers"
	"github.com/ahdark-services/pegasus/pkg/utils"
)

func BindHandlers(r *telegohandler.BotHandler, handlers handlers.Handlers) {
	r.Handle(handlers.ActionCommandHandler, telegohandler.AnyMessageWithText(), telegohandler.Not(utils.PrivateChatOnly()))
}
