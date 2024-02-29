package bot

import (
	"github.com/mymmrac/telego"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

var allowedUpdates = []string{
	"message",
	"edited_message",
	"channel_post",
	"edited_channel_post",
	"message_reaction",
	"message_reaction_count",
	"inline_query",
	"chosen_inline_result",
	"callback_query",
	"shipping_query",
	"pre_checkout_query",
	"poll",
	"poll_answer",
	"my_chat_member",
	"chat_member",
	"chat_join_request",
	"chat_boost",
	"removed_chat_boost",
}

func newWebhookParams(vip *viper.Viper) *telego.SetWebhookParams {
	return &telego.SetWebhookParams{
		URL:                vip.GetString("telegram_bot.webhook.url"),
		IPAddress:          vip.GetString("telegram_bot.webhook.ip_address"),
		MaxConnections:     vip.GetInt("telegram_bot.webhook.max_connections"),
		AllowedUpdates:     allowedUpdates,
		DropPendingUpdates: vip.GetBool("telegram_bot.webhook.drop_pending_updates"),
		SecretToken:        vip.GetString("telegram_bot.webhook.secret_token"),
	}
}

func RegisterWebhook(bot *telego.Bot, params *telego.SetWebhookParams) error {
	if err := bot.SetWebhook(params); err != nil {
		otelzap.L().Error("failed to set webhook", zap.Error(err))
		return err
	}

	return nil
}
