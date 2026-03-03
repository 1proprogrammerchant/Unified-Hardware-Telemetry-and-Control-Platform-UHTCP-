package api

import "net/http"

func ControlHandler(w http.ResponseWriter, r *http.Request) {
    // Accept commands and forward to core
    w.WriteHeader(http.StatusAccepted)
}
