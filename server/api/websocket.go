package api

import (
    "net/http"
)

// WebSocket scaffolding would go here (use gorilla/websocket or stdlib HTTP/2 push).
func WebSocketHandler(w http.ResponseWriter, r *http.Request) {
    w.WriteHeader(501)
}
