package datacenter

import (
	"context"
	"regexp"

	"github.com/ahdark-services/pegasus/pkg/utils"
	"github.com/bytedance/sonic"
	"github.com/imroc/req/v3"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.opentelemetry.io/otel"
	"go.uber.org/zap"
)

var tracer = otel.Tracer("github.com/ahdark-services/pegasus/components/gateway/services/datacenter")

type Service interface {
	QueryDatacenterByUsername(ctx context.Context, username string) (int, error)
}

type service struct {
	client *req.Client
}

func NewService() Service {
	client := req.NewClient().
		SetJsonMarshal(sonic.Marshal).
		SetJsonUnmarshal(sonic.Unmarshal).
		SetBaseURL("https://t.me").
		SetLogger(otelzap.L().Named("service.datacenter").Sugar()).
		SetCommonRetryCount(3).
		SetCommonRetryCondition(func(resp *req.Response, err error) bool {
			return resp.Response.StatusCode >= 500
		}).
		SetCommonRetryHook(func(resp *req.Response, err error) {
			otelzap.L().Ctx(resp.Request.Context()).
				Error("failed to do request",
					zap.Error(err),
					zap.Int("response.status_code", resp.Response.StatusCode),
					zap.String("request.method", resp.Request.Method),
					zap.String("request.url", resp.Request.URL.String()),
				)
		}).
		EnableDumpEachRequest().
		WrapRoundTripFunc(utils.TraceRoundTripWrapperFunc(tracer, "DatacenterService.client.RoundTrip"))

	return &service{client}
}

var dcRegexp = regexp.MustCompile(`https://cdn(\d).cdn-telegram.org/file/[\w-_]+\.\w+`)

func (svc *service) QueryDatacenterByUsername(ctx context.Context, username string) (int, error) {
	ctx, span := tracer.Start(ctx, "DatacenterService.QueryDatacenterByUsername")
	defer span.End()

	resp, err := svc.client.R().SetPathParam("username", username).Get("/{username}")
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to do request", zap.Error(err))
		return 0, err
	}

	if !resp.IsSuccessState() {
		otelzap.L().Ctx(ctx).Error("failed to do request", zap.String("response.body", resp.String()))
		return 0, err
	}

	bodyContent, err := resp.ToString()
	if err != nil {
		otelzap.L().Ctx(ctx).Error("failed to do request", zap.Error(err))
		return 0, err
	}

	matches := dcRegexp.FindStringSubmatch(bodyContent)
	if len(matches) < 2 {
		return 0, nil
	}

	return int(matches[1][0] - '0'), nil
}
