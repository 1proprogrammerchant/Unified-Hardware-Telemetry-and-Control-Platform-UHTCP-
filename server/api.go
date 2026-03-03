package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "log"
    "net/http"
    "os/exec"
    "strings"
    "sync"
    "time"
    "uhtcp/server/internal"
)

func proxyState(w http.ResponseWriter, r *http.Request) {
    client := &http.Client{Timeout: 500 * time.Millisecond}
    resp, err := client.Get("http://127.0.0.1:9000/state")
    if err != nil {
        http.Error(w, "failed to contact core", http.StatusBadGateway)
        return
    }
    defer resp.Body.Close()
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        http.Error(w, "failed to read core response", http.StatusBadGateway)
        return
    }
    // validate JSON from core
    var v interface{}
    if err := json.Unmarshal(body, &v); err != nil {
        http.Error(w, "invalid core JSON", http.StatusBadGateway)
        return
    }
    w.Header().Set("Content-Type", "application/json")
    w.Write(body)
}

func proxyHealth(w http.ResponseWriter, r *http.Request) {
    client := &http.Client{Timeout: 500 * time.Millisecond}
    resp, err := client.Get("http://127.0.0.1:9000/health")
    if err != nil {
        http.Error(w, "failed to contact core", http.StatusBadGateway)
        return
    }
    defer resp.Body.Close()
    body, err := io.ReadAll(resp.Body)
    if err != nil {
        http.Error(w, "failed to read core response", http.StatusBadGateway)
        return
    }
    // validate JSON from core
    var v interface{}
    if err := json.Unmarshal(body, &v); err != nil {
        http.Error(w, "invalid core JSON", http.StatusBadGateway)
        return
    }
    w.Header().Set("Content-Type", "application/json")
    w.Write(body)
}

func main() {
    ipc := internal.NewIPCClient("http://127.0.0.1:9000")
    ipc.Start()
    defer ipc.Stop()

    http.HandleFunc("/api/v1/state", proxyState)
    http.HandleFunc("/api/v1/health", proxyHealth)
    http.HandleFunc("/api/v1/scripts/start", handleScriptStart)
    http.HandleFunc("/api/v1/scripts/stop", handleScriptStop)
    fmt.Println("API server listening on :8080")
    http.ListenAndServe(":8080", nil)
}

// Simple in-memory supervisor for Ruby runners
type ScriptProc struct {
    cmd *exec.Cmd
    stdin io.WriteCloser
    stdout io.ReadCloser
}

var (
    scripts   = map[string]*ScriptProc{}
    scriptsMu sync.Mutex
)

func handleScriptStart(w http.ResponseWriter, r *http.Request) {
    type reqT struct{ Path string `json:"path"`; ID string `json:"id"` }
    var req reqT
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, "bad request", http.StatusBadRequest)
        return
    }
    if req.Path == "" || req.ID == "" {
        http.Error(w, "missing fields", http.StatusBadRequest)
        return
    }
    scriptsMu.Lock()
    defer scriptsMu.Unlock()
    if _, ok := scripts[req.ID]; ok {
        http.Error(w, "id exists", http.StatusConflict)
        return
    }
    cmd := exec.Command("ruby", "automation/sandbox/runner.rb", req.Path)
    stdin, err := cmd.StdinPipe()
    if err != nil { http.Error(w, "internal", 500); return }
    stdout, err := cmd.StdoutPipe()
    if err != nil { http.Error(w, "internal", 500); return }
    if err := cmd.Start(); err != nil { http.Error(w, "failed to start", 500); return }
    sp := &ScriptProc{cmd: cmd, stdin: stdin, stdout: stdout}
    scripts[req.ID] = sp

    // Start reader for stdout actions
    go func(id string, sp *ScriptProc) {
        buf := make([]byte, 4096)
        for {
            n, err := sp.stdout.Read(buf)
            if err != nil { return }
            // split lines
            for _, line := range splitLines(buf[:n]) {
                var m map[string]interface{}
                if err := json.Unmarshal([]byte(line), &m); err == nil {
                    if m["type"] == "action" && m["action"] == "write_gpio" {
                        pin := int(m["pin"].(float64))
                        val := int(m["value"].(float64))
                        // forward to Rust core control endpoint
                        forwardGPIO(pin, val)
                    }
                }
            }
        }
    }(req.ID, sp)

    w.WriteHeader(http.StatusCreated)
}

func handleScriptStop(w http.ResponseWriter, r *http.Request) {
    type reqT struct{ ID string `json:"id"` }
    var req reqT
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, "bad request", http.StatusBadRequest)
        return
    }
    scriptsMu.Lock()
    defer scriptsMu.Unlock()
    sp, ok := scripts[req.ID]
    if !ok { http.Error(w, "not found", http.StatusNotFound); return }
    sp.cmd.Process.Kill()
    delete(scripts, req.ID)
    w.WriteHeader(http.StatusOK)
}

func splitLines(b []byte) []string {
    var out []string
    s := string(b)
    for _, line := range strings.Split(s, "\n") {
        if len(line) > 0 { out = append(out, line) }
    }
    return out
}

func forwardGPIO(pin, value int) {
    client := &http.Client{Timeout: 200 * time.Millisecond}
    body := map[string]int{"pin": pin, "value": value}
    b, _ := json.Marshal(body)
    // prefer async PostJSON; fall back to direct POST when queue is full
    if ipcAsync := internal.NewIPCClient("http://127.0.0.1:9000"); ipcAsync != nil {
        // Note: use a short-lived client here for compatibility with older callers
        respCh, err := ipcAsync.PostJSON("/control/gpio", b, 200*time.Millisecond)
        if err == nil {
            go func() {
                resp := <-respCh
                if resp.Err != nil {
                    log.Printf("forwardGPIO async error: %v", resp.Err)
                }
            }()
            return
        }
    }
    client.Post("http://127.0.0.1:9000/control/gpio", "application/json", bytes.NewReader(b))
}
