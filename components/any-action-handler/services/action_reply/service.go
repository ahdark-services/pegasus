package action_reply

import (
	"context"
	"fmt"
	"github.com/mymmrac/telego"
	"github.com/pkg/errors"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/fx"
	"strings"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/any-action-handler/internal/services/action_reply")

type Service interface {
	CheckNeedReply(ctx context.Context, action string) bool
	GetReplyTemplate(ctx context.Context, action string, sender, user *telego.User) (string, error)
}

type service struct {
	fx.In
}

func NewService(s service) Service {
	return &s
}

func (svc *service) CheckNeedReply(ctx context.Context, action string) bool {
	ctx, span := tracer.Start(ctx, "ActionReplyService.CheckNeedReply", trace.WithAttributes(
		attribute.String("action", action),
	))
	defer span.End()

	action = strings.TrimPrefix(action, "/")
	if action == "" {
		return false
	}

	return checkIfChinese([]rune(action)[0]) || strings.HasPrefix(action, "$")
}

func (svc *service) GetReplyTemplate(ctx context.Context, action string, sender, user *telego.User) (string, error) {
	ctx, span := tracer.Start(ctx, "ActionReplyService.GetReplyTemplate")
	defer span.End()

	action = strings.TrimPrefix(action, "/")

	sections := strings.Split(action, " ")
	switch len(sections) {
	case 0:
		return "", errors.New("action is empty")
	case 1:
		return fmt.Sprintf(`<a href="tg://user?id=%d">%s</a> %s了 <a href="tg://user?id=%d">%s</a>`, sender.ID, getFullUserNickname(sender), sections[0], user.ID, getFullUserNickname(user)), nil
	default:
		return fmt.Sprintf(`<a href="tg://user?id=%d">%s</a> %s <a href="tg://user?id=%d">%s</a> %s`, sender.ID, getFullUserNickname(sender), sections[0], user.ID, getFullUserNickname(user), strings.Join(sections[1:], " ")), nil
	}
}
