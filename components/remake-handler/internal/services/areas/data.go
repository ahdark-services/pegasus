package areas

import (
	_ "embed"
	"reflect"

	"github.com/bytedance/sonic"
)

//go:embed areas.json
var areasData []byte

type AreasData []AreasDatum

func UnmarshalAreasData(data []byte) (AreasData, error) {
	var r AreasData
	err := sonic.Unmarshal(data, &r)
	return r, err
}

func (r *AreasData) Marshal() ([]byte, error) {
	return sonic.Marshal(r)
}

type AreasDatum struct {
	Country string   `json:"country"`
	Cities  []string `json:"cities"`
}

func init() {
	if err := sonic.Pretouch(reflect.TypeOf(AreasDatum{})); err != nil {
		panic(err)
		return
	}
}
