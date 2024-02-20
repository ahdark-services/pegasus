package serializer

import (
	"errors"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestNewAppResponse(t *testing.T) {
	asserts := assert.New(t)

	asserts.Equal(&AppResponse{
		Code:    CodeSuccess,
		Message: "success",
		Data:    nil,
	}, NewAppResponse(CodeSuccess, "success", nil))
}

func TestNewAppResponseErrorMsg(t *testing.T) {
	asserts := assert.New(t)

	asserts.Equal(&AppResponse{
		Code:    CodeErrInvalidRequest,
		Message: "error",
		Data:    nil,
	}, NewAppResponseErrorMsg(CodeErrInvalidRequest, "error"))
}

func TestNewAppResponseError(t *testing.T) {
	asserts := assert.New(t)

	asserts.Equal(&AppResponse{
		Code:    CodeErrInvalidRequest,
		Message: "error",
		Data:    nil,
	}, NewAppResponseError(CodeErrInvalidRequest, errors.New("error")))
}

func TestNewAppResponseSuccess(t *testing.T) {
	asserts := assert.New(t)

	asserts.Equal(&AppResponse{
		Code:    CodeSuccess,
		Message: "success",
		Data:    nil,
	}, NewAppResponseSuccess(nil))
}
