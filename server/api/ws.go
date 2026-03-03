package api

import (
    "net/http"

    "uhtcp/server/internal"

    "github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}

func WSHandler(w http.ResponseWriter, r *http.Request) {
    conn, err := upgrader.Upgrade(w, r, nil)
    if err != nil {
        http.Error(w, "upgrade failed", http.StatusBadRequest)
        return
    }
    internal.NewWSClient(conn)
}
