package entities

import (
	"encoding/gob"
	"github.com/bytedance/sonic"
	"reflect"
)

type Address struct {
	Country  string `json:"country"`  // Country, Nation, etc.
	State    string `json:"state"`    // Province, Region, etc.
	City     string `json:"city"`     // City, Town, etc.
	Address1 string `json:"address1"` // Street, Road, etc.
	Address2 string `json:"address2"` // Apartment, Suite, etc.
	ZipCode  string `json:"zip_code"` // Postal Code, Zip Code, etc.
}

func init() {
	gob.Register(&Address{})
	if err := sonic.Pretouch(reflect.TypeOf(&Address{})); err != nil {
		panic(err)
	}
}
