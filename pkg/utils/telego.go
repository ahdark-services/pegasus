package utils

import (
	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegohandler"
)

func PrivateChatOnly() telegohandler.Predicate {
	return func(update telego.Update) bool {
		if update.Message != nil {
			return update.Message.Chat.Type == telego.ChatTypePrivate
		}

		return false
	}
}
