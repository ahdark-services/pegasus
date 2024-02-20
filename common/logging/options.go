package logging

import (
	"context"

	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/zap"
)

func callerOption(v *viper.Viper) zap.Option {
	return zap.WithCaller(v.GetBool("logging.caller"))
}

func stacktraceOption(ctx context.Context, v *viper.Viper) (zap.Option, error) {
	ctx, span := tracer.Start(ctx, "logging.stacktraceOption")
	defer span.End()

	level, err := zap.ParseAtomicLevel(v.GetString("logging.stacktrace"))
	if err != nil {
		otelzap.Ctx(ctx).Error("failed to parse stacktrace level", zap.Error(err))
		return nil, err
	}

	return zap.AddStacktrace(level), err
}
