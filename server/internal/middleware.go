package internal

import (
    "log"
    "net"
    "net/http"
    "sync"
    "time"
)

func LoggingMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        start := time.Now()
        next.ServeHTTP(w, r)
        log.Printf("%s %s %s", r.Method, r.URL.Path, time.Since(start))
    })
}

// Simple global rate limiter (per remote IP) - token refill based
var (
    limits   = map[string]*tokenBucket{}
    limitsMu sync.Mutex
)

type tokenBucket struct {
    mu         sync.Mutex
    tokens     float64
    last       time.Time
    ratePerSec float64
    capacity   float64
}

func newBucket(ratePerMin float64) *tokenBucket {
    return &tokenBucket{tokens: ratePerMin, last: time.Now(), ratePerSec: ratePerMin / 60.0, capacity: ratePerMin}
}

func (b *tokenBucket) allow() bool {
    b.mu.Lock()
    defer b.mu.Unlock()
    now := time.Now()
    elapsed := now.Sub(b.last).Seconds()
    b.tokens += elapsed * b.ratePerSec
    if b.tokens > b.capacity {
        b.tokens = b.capacity
    }
    b.last = now
    if b.tokens >= 1.0 {
        b.tokens -= 1.0
        return true
    }
    return false
}

func RateLimitMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        ip, _, _ := net.SplitHostPort(r.RemoteAddr)
        limitsMu.Lock()
        b, ok := limits[ip]
        if !ok {
            b = newBucket(60) // 60 req/min by default
            limits[ip] = b
        }
        limitsMu.Unlock()
        if !b.allow() {
            http.Error(w, "rate limit", http.StatusTooManyRequests)
            return
        }
        next.ServeHTTP(w, r)
    })
}
