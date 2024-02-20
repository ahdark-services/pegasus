package dcron

import (
	"github.com/libi/dcron/dlog"
	"go.uber.org/zap"
)

type Logger struct {
	*zap.Logger
}

func newLogger(l *zap.Logger) dlog.Logger {
	return &Logger{l}
}

func (l *Logger) Printf(s string, a ...any) {
	l.Sugar().Debugf(s, a...)
}

func (l *Logger) Infof(s string, a ...any) {
	l.Sugar().Infof(s, a...)
}

func (l *Logger) Warnf(s string, a ...any) {
	l.Sugar().Warnf(s, a...)
}

func (l *Logger) Errorf(s string, a ...any) {
	l.Sugar().Errorf(s, a...)
}
