package api

import "net/http"

func Routes() http.Handler {
    mux := http.NewServeMux()
    mux.HandleFunc("/api/v1/state", func(w http.ResponseWriter, r *http.Request) { w.Write([]byte("{}")) })
    return mux
}
