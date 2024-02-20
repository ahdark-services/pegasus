package server

import (
	"context"
	"net/http"

	"github.com/cloudwego/hertz/pkg/app"

	"github.com/ahdark-services/pegasus/common/serializer"
)

func HealthHandler(ctx context.Context, c *app.RequestContext) {
	ctx, span := tracer.Start(ctx, "server.HealthHandler")
	defer span.End()

	serializer.NewAppResponseSuccess(nil).JSON(c)
}

func NoRouteHandler(ctx context.Context, c *app.RequestContext) {
	ctx, span := tracer.Start(ctx, "server.NoRouteHandler")
	defer span.End()

	serializer.NewAppResponseErrorMsg(serializer.CodeErrNotFound, "no route").
		AbortWithStatusJSON(c, http.StatusNotFound)
}

func NoMethodHandler(ctx context.Context, c *app.RequestContext) {
	ctx, span := tracer.Start(ctx, "server.NoMethodHandler")
	defer span.End()

	serializer.NewAppResponseErrorMsg(serializer.CodeErrNotFound, "method not allowed").
		AbortWithStatusJSON(c, http.StatusMethodNotAllowed)
}
