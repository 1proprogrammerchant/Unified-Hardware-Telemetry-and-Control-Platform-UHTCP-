package internal

import (
    "bytes"
    "context"
    "errors"
    "io"
    "math"
    "net/http"
    "sync"
    "time"
)

type IPCResponse struct {
    Body []byte
    Err  error
}

type IPCRequest struct {
    Method  string
    Path    string
    Body    []byte
    Timeout time.Duration
    RespCh  chan IPCResponse
}

type IPCClient struct {
    baseURL    string
    client     *http.Client
    queue      chan *IPCRequest
    wg         sync.WaitGroup
    mu         sync.Mutex
    closed     bool
    maxWorkers int
}

func NewIPCClient(baseURL string) *IPCClient {
    return &IPCClient{
        baseURL:    baseURL,
        client:     &http.Client{Timeout: 5 * time.Second},
        queue:      make(chan *IPCRequest, 1024),
        maxWorkers: 4,
    }
}

// Start launches background workers to process queued requests.
func (c *IPCClient) Start() {
    for i := 0; i < c.maxWorkers; i++ {
        c.wg.Add(1)
        go func() {
            defer c.wg.Done()
            c.workerLoop()
        }()
    }
}

// Stop signals workers to finish and waits for them.
func (c *IPCClient) Stop() {
    // Close the queue to stop accepting new requests and let workers drain it.
    c.mu.Lock()
    if c.closed {
        c.mu.Unlock()
        return
    }
    c.closed = true
    close(c.queue)
    c.mu.Unlock()
    c.wg.Wait()
}

// Enqueue submits a request to the background queue. Returns an error if the queue is full.
func (c *IPCClient) Enqueue(req *IPCRequest) (err error) {
    c.mu.Lock()
    if c.closed {
        c.mu.Unlock()
        return errors.New("ipc client stopped")
    }
    c.mu.Unlock()

    // If the queue is closed right after the check above, a send will panic.
    // Recover and return a friendly error instead.
    defer func() {
        if r := recover(); r != nil {
            err = errors.New("ipc client stopped")
        }
    }()

    select {
    case c.queue <- req:
        return nil
    default:
        return errors.New("ipc queue full")
    }
}

// PostJSON posts JSON to the core asynchronously and returns a response channel.
func (c *IPCClient) PostJSON(path string, body []byte, timeout time.Duration) (chan IPCResponse, error) {
    respCh := make(chan IPCResponse, 1)
    req := &IPCRequest{Method: "POST", Path: path, Body: body, Timeout: timeout, RespCh: respCh}
    if err := c.Enqueue(req); err != nil {
        return nil, err
    }
    return respCh, nil
}

// Get performs a synchronous GET (convenience wrapper) with timeout.
func (c *IPCClient) Get(path string, timeout time.Duration) ([]byte, error) {
    ctx, cancel := context.WithTimeout(context.Background(), timeout)
    defer cancel()
    req, err := http.NewRequestWithContext(ctx, "GET", c.baseURL+path, nil)
    if err != nil {
        return nil, err
    }
    resp, err := c.client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    return io.ReadAll(resp.Body)
}

func (c *IPCClient) workerLoop() {
    // Range over the queue channel so workers process all queued requests
    // until the channel is closed by Stop(). This ensures graceful shutdown
    // with no abrupt return.
    for req := range c.queue {
        if req == nil {
            continue
        }
        c.handleWithRetry(req)
    }
}

func (c *IPCClient) handleWithRetry(r *IPCRequest) {
    const maxAttempts = 4
    var lastErr error
    for attempt := 1; attempt <= maxAttempts; attempt++ {
        body, err := c.doRequest(r)
        if err == nil {
            r.RespCh <- IPCResponse{Body: body, Err: nil}
            return
        }
        lastErr = err
        // exponential backoff with jitter
        backoff := time.Duration(math.Exp2(float64(attempt))) * 100 * time.Millisecond
        jitter := time.Duration((float64(backoff) * 0.2) * (0.5 + randFloat()))
        time.Sleep(backoff + jitter)
    }
    r.RespCh <- IPCResponse{Body: nil, Err: lastErr}
}

func (c *IPCClient) doRequest(r *IPCRequest) ([]byte, error) {
    ctx := context.Background()
    if r.Timeout > 0 {
        var cancel context.CancelFunc
        ctx, cancel = context.WithTimeout(ctx, r.Timeout)
        defer cancel()
    }
    var bodyReader io.Reader
    if r.Body != nil {
        bodyReader = bytes.NewReader(r.Body)
    }
    req, err := http.NewRequestWithContext(ctx, r.Method, c.baseURL+r.Path, bodyReader)
    if err != nil {
        return nil, err
    }
    if r.Body != nil {
        req.Header.Set("Content-Type", "application/json")
    }
    resp, err := c.client.Do(req)
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    if resp.StatusCode >= 400 {
        b, _ := io.ReadAll(resp.Body)
        return nil, errors.New(string(b))
    }
    return io.ReadAll(resp.Body)
}

// simple pseudo-random float [0,1)
func randFloat() float64 {
    return float64(time.Now().UnixNano()%1000) / 1000.0
}

 