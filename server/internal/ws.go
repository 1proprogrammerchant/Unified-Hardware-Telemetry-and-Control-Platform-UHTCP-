package internal

import (
    "log"
    "sync"
    "time"

    "github.com/gorilla/websocket"
)

type WSClient struct {
    conn *websocket.Conn
    send chan []byte
}

func NewWSClient(conn *websocket.Conn) *WSClient {
    c := &WSClient{conn: conn, send: make(chan []byte, 256)}
    DefaultHub.Register(c)
    go c.readPump()
    go c.writePump()
    return c
}

func (c *WSClient) readPump() {
    defer func() { c.conn.Close() }()
    c.conn.SetReadLimit(1024)
    for {
        _, msg, err := c.conn.ReadMessage()
        if err != nil {
            return
        }
        // broadcast what we received
        DefaultHub.Broadcast(msg)
    }
}

func (c *WSClient) writePump() {
    defer func() { c.conn.Close() }()
    ticker := time.NewTicker(30 * time.Second)
    defer ticker.Stop()
    for {
        select {
        case msg, ok := <-c.send:
            if !ok {
                return
            }
            if err := c.conn.WriteMessage(websocket.TextMessage, msg); err != nil {
                return
            }
        case <-ticker.C:
            if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
                return
            }
        }
    }
}

// Hub
type Hub struct {
    mu        sync.Mutex
    clients   map[*WSClient]struct{}
    broadcast chan []byte
}

var DefaultHub = NewHub()

func NewHub() *Hub {
    h := &Hub{clients: make(map[*WSClient]struct{}), broadcast: make(chan []byte, 256)}
    go h.run()
    return h
}

func (h *Hub) run() {
    for msg := range h.broadcast {
        h.mu.Lock()
        for c := range h.clients {
            select {
            case c.send <- msg:
            default:
                close(c.send)
                delete(h.clients, c)
            }
        }
        h.mu.Unlock()
    }
}

func (h *Hub) Register(c *WSClient) {
    h.mu.Lock()
    h.clients[c] = struct{}{}
    h.mu.Unlock()
}

func (h *Hub) Broadcast(msg []byte) {
    select {
    case h.broadcast <- msg:
    default:
        // drop if hub queue full
        log.Println("ws: drop broadcast, hub full")
    }
}
