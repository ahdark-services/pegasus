package serializer

import (
	"github.com/cloudwego/hertz/pkg/app"
	"net/http"
)

type AppResponse struct {
	Code    ResponseCode `json:"code"`
	Message string       `json:"message"`
	Data    interface{}  `json:"data,omitempty"`
}

func NewAppResponse(code ResponseCode, message string, data interface{}) *AppResponse {
	return &AppResponse{
		Code:    code,
		Message: message,
		Data:    data,
	}
}

func NewAppResponseErrorMsg(code ResponseCode, message string) *AppResponse {
	return NewAppResponse(code, message, nil)
}

func NewAppResponseError(code ResponseCode, err error) *AppResponse {
	return NewAppResponse(code, err.Error(), nil)
}

func NewAppResponseSuccess(data interface{}) *AppResponse {
	return NewAppResponse(CodeSuccess, "success", data)
}

func (r *AppResponse) AbortWithStatusJSON(c *app.RequestContext, code int) {
	c.AbortWithStatusJSON(code, r)
}

func (r *AppResponse) JSON(c *app.RequestContext) {
	r.JSONWithStatus(c, http.StatusOK)
}

func (r *AppResponse) JSONWithStatus(c *app.RequestContext, code int) {
	c.JSON(code, r)
}
