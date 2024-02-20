package serializer

type ResponseCode int

const (
	CodeSuccess ResponseCode = iota
)

// 1xxx: common error
const (
	CodeErrInvalidRequest ResponseCode = iota + 1000
	CodeErrInvalidParameter
	CodeErrInvalidToken
	CodeErrNotFound
	CodeErrUnauthorized
	CodeErrRateLimited
)

// 2xxx: service error
const (
	CodeErrServerError ResponseCode = iota + 2000
	CodeErrServiceError
)
