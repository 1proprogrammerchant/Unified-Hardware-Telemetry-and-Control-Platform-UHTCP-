package internal

import "encoding/json"

func ToJSON(v interface{}) ([]byte, error) { return json.Marshal(v) }
