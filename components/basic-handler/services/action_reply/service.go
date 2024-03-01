package action_reply

import (
	"context"
	_ "embed"
	"strings"
	"text/template"

	"github.com/cloudwego/hertz/pkg/common/bytebufferpool"
	"github.com/mymmrac/telego"
	"github.com/pkg/errors"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/fx"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/basic-handler/internal/services/action_reply")

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

var funcMap = template.FuncMap{
	"getFullUserNickname": getFullUserNickname,
}

//go:embed reply.tpl
var replyTemplateText string

var replyTemplate = template.Must(template.New("reply").Funcs(funcMap).Parse(replyTemplateText))

type replyData struct {
	Sender  *telego.User
	User    *telego.User
	Action  string
	Message string
}

func (svc *service) GetReplyTemplate(ctx context.Context, action string, sender, user *telego.User) (string, error) {
	ctx, span := tracer.Start(ctx, "ActionReplyService.GetReplyTemplate")
	defer span.End()

	action = strings.TrimSpace(action)
	action = strings.TrimPrefix(action, "/")
	action = strings.TrimPrefix(action, "$")

	sections := strings.Split(action, " ")
	switch len(sections) {
	case 0:
		otelzap.L().Ctx(ctx).Error("action is empty")
		return "", errors.New("action is empty")
	case 1:
		buf := bytebufferpool.Get()
		defer bytebufferpool.Put(buf)

		if err := replyTemplate.Execute(buf, replyData{
			Sender: sender,
			User:   user,
			Action: sections[0],
		}); err != nil {
			otelzap.L().Ctx(ctx).Error("failed to execute template", zap.Error(err))
			return "", errors.Wrap(err, "failed to execute template")
		}

		return buf.String(), nil
	default:
		buf := bytebufferpool.Get()
		defer bytebufferpool.Put(buf)

		if err := replyTemplate.Execute(buf, replyData{
			Sender:  sender,
			User:    user,
			Action:  sections[0],
			Message: strings.Join(sections[1:], " "),
		}); err != nil {
			otelzap.L().Ctx(ctx).Error("failed to execute template", zap.Error(err))
			return "", errors.Wrap(err, "failed to execute template")
		}

		return buf.String(), nil
	}
}
