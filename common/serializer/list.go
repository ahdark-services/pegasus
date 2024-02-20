package serializer

type ListResponse[T any] struct {
	Items []*T  `json:"items"`
	Count int64 `json:"count"`
}
