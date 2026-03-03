package main

import (
    "fmt"
    "net/http"
)

func main() {
    resp, err := http.Get("http://127.0.0.1:8080/api/v1/state")
    if err != nil { fmt.Println("err", err); return }
    defer resp.Body.Close()
    fmt.Println("status", resp.Status)
}
