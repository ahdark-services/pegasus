package datacenter

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNewService(t *testing.T) {
	asserts := assert.New(t)
	asserts.NotNil(NewService())
}

func TestService_GetDatacenter(t *testing.T) {
	asserts := assert.New(t)
	svc := NewService()

	dc, err := svc.QueryDatacenterByUsername(context.Background(), "durov")
	asserts.NoError(err)
	asserts.Equal(1, dc)
}
