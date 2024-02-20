package utils

import (
	"encoding/gob"
	"github.com/stretchr/testify/assert"
	"testing"
)

type TestStruct struct {
	A string
}

func TestMain(m *testing.M) {
	gob.Register(TestStruct{})

	m.Run()
}

func TestGobEncode(t *testing.T) {
	asserts := assert.New(t)

	data := TestStruct{
		A: "test",
	}

	d, err := GobEncode(data)
	asserts.NoError(err)
	asserts.Equal([]byte{29, 127, 3, 1, 1, 10, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 1, 255, 128, 0, 1, 1, 1, 1, 65, 1, 12, 0, 0, 0, 9, 255, 128, 1, 4, 116, 101, 115, 116, 0}, d)
}

func TestGobDecode(t *testing.T) {
	asserts := assert.New(t)

	data := []byte{29, 127, 3, 1, 1, 10, 84, 101, 115, 116, 83, 116, 114, 117, 99, 116, 1, 255, 128, 0, 1, 1, 1, 1, 65, 1, 12, 0, 0, 0, 9, 255, 128, 1, 4, 116, 101, 115, 116, 0}

	var d TestStruct
	err := GobDecode(data, &d)
	asserts.NoError(err)
	asserts.Equal(TestStruct{
		A: "test",
	}, d)
}
