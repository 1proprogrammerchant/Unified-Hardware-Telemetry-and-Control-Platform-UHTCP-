package main

import (
    "log"
    "net/http"
    "time"

    "uhtcp/server/api"
    "uhtcp/server/internal"
)

func main() {
    ipc := internal.NewIPCClient("http://127.0.0.1:9000")
    ipc.Start()

    mux := http.NewServeMux()
    mux.HandleFunc("/api/v1/health", api.HealthHandler(ipc))
    mux.Handle("/metrics", internal.MetricsHandler())
    mux.HandleFunc("/ws", api.WSHandler)

    handler := internal.LoggingMiddleware(internal.RateLimitMiddleware(mux))

    srv := &http.Server{
        Addr:         ":8080",
        Handler:      handler,
        ReadTimeout:  5 * time.Second,
        WriteTimeout: 10 * time.Second,
        IdleTimeout:  120 * time.Second,
    }

    log.Println("server listening :8080")
    if err := srv.ListenAndServe(); err != nil {
        log.Fatal(err)
    }
}
