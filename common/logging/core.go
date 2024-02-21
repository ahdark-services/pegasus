package logging

import (
	"context"
	"os"

	"github.com/ahdark-services/pegasus/pkg/utils"
	"github.com/spf13/viper"
	"github.com/uptrace/opentelemetry-go-extra/otelzap"
	"go.uber.org/fx"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"gopkg.in/natefinch/lumberjack.v2"
)

type coreConfig struct {
	Encoder string `yaml:"encoder"` // text, json
	Target  string `yaml:"target"`  // stdout, stderr, file, fluent
	Level   string `yaml:"level"`
	Sync    bool   `yaml:"sync,omitempty"`

	Filepath string `yaml:"filepath,omitempty"`
}

func (c coreConfig) GetEncoder(config zapcore.EncoderConfig) zapcore.Encoder {
	switch c.Encoder {
	case "text", "console":
		return zapcore.NewConsoleEncoder(config)
	case "json":
		return zapcore.NewJSONEncoder(config)
	default:
		otelzap.L().Warn("unknown encoder", zap.String("encoder", c.Encoder))
		return zapcore.NewConsoleEncoder(config)
	}
}

func (c coreConfig) GetLevel() (zap.AtomicLevel, error) {
	level, err := zap.ParseAtomicLevel(c.Level)
	if err != nil {
		return zap.NewAtomicLevelAt(zapcore.InfoLevel), err
	}

	return level, err
}

func coreOption(ctx context.Context, v *viper.Viper, lc fx.Lifecycle) (zap.Option, error) {
	ctx, span := tracer.Start(ctx, "logging.coreOption")
	defer span.End()

	encoderConfig := zapcore.EncoderConfig{
		MessageKey:     "msg",
		LevelKey:       "level",
		TimeKey:        "timestamp",
		NameKey:        "logger",
		CallerKey:      "caller",
		FunctionKey:    "func",
		StacktraceKey:  "stack",
		LineEnding:     zapcore.DefaultLineEnding,
		EncodeLevel:    zapcore.CapitalLevelEncoder,
		EncodeTime:     zapcore.RFC3339TimeEncoder,
		EncodeDuration: zapcore.StringDurationEncoder,
		EncodeCaller:   zapcore.FullCallerEncoder,
		EncodeName:     zapcore.FullNameEncoder,
	}

	var configs []coreConfig
	if err := v.UnmarshalKey("logging.core", &configs); err != nil {
		otelzap.L().Ctx(ctx).Panic("failed to unmarshal logging core config", zap.Error(err))
		return nil, err
	}

	var cores []zapcore.Core
	for _, config := range configs {
		level, err := config.GetLevel()
		if err != nil {
			otelzap.L().Ctx(ctx).Error("failed to get logging level", zap.Error(err))
			return nil, err
		}

		var writeSyncer zapcore.WriteSyncer
		switch config.Target {
		case "stdout":
			writeSyncer = os.Stdout
		case "stderr":
			writeSyncer = os.Stderr
		case "file":
			path := utils.AbsolutePath(config.Filepath)
			if _, err := utils.CreateIfNotExist(path); err != nil {
				otelzap.L().Ctx(ctx).Error("failed to create log file", zap.Error(err))
				return nil, err
			}

			f := &lumberjack.Logger{Filename: path}
			writeSyncer = zapcore.AddSync(f)
			lc.Append(fx.Hook{
				OnStop: func(ctx context.Context) error {
					ctx, span := tracer.Start(ctx, "logging.coreOption.OnStop")
					defer span.End()

					if err := f.Rotate(); err != nil {
						span.RecordError(err)
						otelzap.L().Ctx(ctx).Error("failed to rotate log file", zap.Error(err))
						return err
					}

					return nil
				},
			})
		default:
			otelzap.L().Ctx(ctx).Panic("unknown logging target", zap.String("target", config.Target))
			continue
		}

		if config.Sync {
			writeSyncer = zapcore.Lock(writeSyncer)
		}

		core := zapcore.NewCore(
			config.GetEncoder(encoderConfig),
			writeSyncer,
			level,
		)

		cores = append(cores, core)
		otelzap.L().Ctx(ctx).Info("add logging core",
			zap.String("target", config.Target),
			zap.String("level", config.Level),
			zap.String("encoder", config.Encoder),
		)
	}

	return zap.WrapCore(func(core zapcore.Core) zapcore.Core {
		return zapcore.NewTee(cores...)
	}), nil
}
