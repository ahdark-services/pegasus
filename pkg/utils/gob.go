package utils

import (
	"bytes"
	"encoding/gob"

	"github.com/cloudwego/hertz/pkg/common/bytebufferpool"
)

func GobEncode(v interface{}) ([]byte, error) {
	b := bytebufferpool.Get()
	defer bytebufferpool.Put(b)

	if err := gob.NewEncoder(b).Encode(v); err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}

func GobDecode(b []byte, v interface{}) error {
	return gob.NewDecoder(bytes.NewReader(b)).Decode(v)
}
