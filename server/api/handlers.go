package api

import (
    "encoding/json"
    "net/http"
    "time"
)

type ipcClient interface {
    Get(path string, timeout time.Duration) ([]byte, error)
}

func HealthHandler(ipc ipcClient) http.HandlerFunc {
    return func(w http.ResponseWriter, r *http.Request) {
        body, err := ipc.Get("/health", 500*time.Millisecond)
        if err != nil {
            http.Error(w, "failed to contact core", http.StatusBadGateway)
            return
        }
        var v interface{}
        if err := json.Unmarshal(body, &v); err != nil {
            http.Error(w, "invalid core JSON", http.StatusBadGateway)
            return
        }
        w.Header().Set("Content-Type", "application/json")
        w.Write(body)
    }
}
