package action_reply

import (
	"context"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestService_CheckNeedReply(t *testing.T) {
	asserts := assert.New(t)
	svc := NewService(service{})

	asserts.False(svc.CheckNeedReply(context.Background(), "/test"))
	asserts.True(svc.CheckNeedReply(context.Background(), "/$test"))
	asserts.True(svc.CheckNeedReply(context.Background(), "超"))
	asserts.True(svc.CheckNeedReply(context.Background(), "/超"))
}
