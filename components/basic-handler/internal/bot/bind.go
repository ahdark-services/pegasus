package bot

import (
	"github.com/ahdark-services/pegasus/components/basic-handler/internal/bot/handlers"
	"github.com/ahdark-services/pegasus/pkg/utils"
	"github.com/mymmrac/telego/telegohandler"
)

func BindHandlers(r *telegohandler.BotHandler, handlers handlers.Handlers) {
	r.Handle(handlers.RemakeCommandHandler, telegohandler.And(telegohandler.CommandEqual("start"), utils.PrivateChatOnly()))
}
