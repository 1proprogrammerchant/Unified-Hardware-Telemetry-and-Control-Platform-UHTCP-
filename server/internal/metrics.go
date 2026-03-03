package internal

import (
    "net/http"

    "github.com/prometheus/client_golang/prometheus"
    "github.com/prometheus/client_golang/prometheus/promhttp"
)

func init() {
    // Register standard collectors
    prometheus.MustRegister(prometheus.NewProcessCollector(prometheus.ProcessCollectorOpts{}))
    prometheus.MustRegister(prometheus.NewGoCollector())
}

// MetricsHandler returns an http.Handler exposing Prometheus metrics.
func MetricsHandler() http.Handler {
    return promhttp.Handler()
}
