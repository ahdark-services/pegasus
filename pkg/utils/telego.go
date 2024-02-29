package utils

import (
	"github.com/mymmrac/telego"
	"github.com/mymmrac/telego/telegohandler"
)

func PrivateChatOnly() telegohandler.Predicate {
	return StrictChatType(telego.ChatTypePrivate)
}

func GroupChatOnly() telegohandler.Predicate {
	return StrictChatType(telego.ChatTypeGroup)
}

func StrictChatType(chatType string) telegohandler.Predicate {
	return func(update telego.Update) bool {
		if update.Message != nil {
			return update.Message.Chat.Type == chatType
		}

		return false
	}
}
