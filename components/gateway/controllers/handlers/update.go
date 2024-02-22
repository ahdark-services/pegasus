package handlers

import (
	"context"
	"github.com/ahdark-services/pegasus/common/serializer"
	"github.com/cloudwego/hertz/pkg/app"
	"github.com/mymmrac/telego"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
	"net/http"
)

type UpdateRequest struct {
	telego.Update
}

func (h *handlers) UpdateHandler(ctx context.Context, c *app.RequestContext) {
	ctx, span := tracer.Start(ctx, "handlers.UpdateHandler")
	defer span.End()

	var update UpdateRequest
	if err := c.Bind(&update); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to bind update", zap.Error(err))
		serializer.NewAppResponseError(serializer.CodeErrInvalidParameter, err).
			AbortWithStatusJSON(c, http.StatusBadRequest)
		return
	}

	if err := h.TransportService.SendUpdate(ctx, update.Update); err != nil {
		otelzap.L().Ctx(ctx).Error("failed to send update", zap.Error(err))
		serializer.NewAppResponseError(serializer.CodeErrServiceError, err).
			AbortWithStatusJSON(c, http.StatusInternalServerError)
		return
	}

	serializer.NewAppResponseSuccess(nil).JSON(c)
}
