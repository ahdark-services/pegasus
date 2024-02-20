package utils

import (
	bs "github.com/AH-dark/bytestring/v2"
	"github.com/bytedance/sonic"
)

func MustMarshalJSON(v interface{}) []byte {
	b, err := sonic.Marshal(v)
	if err != nil {
		panic(err)
	}

	return b
}

func MustMarshalString(v interface{}) string {
	return bs.B2S(MustMarshalJSON(v))
}
