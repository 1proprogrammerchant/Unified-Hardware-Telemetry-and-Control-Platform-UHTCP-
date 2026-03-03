#!/usr/bin/env python3
import json
from http.server import BaseHTTPRequestHandler, HTTPServer
import threading
import time

state = {
    "cpu_temperature": 42.0,
    "uptime": 0,
    "gpio_state": 0
}

class Handler(BaseHTTPRequestHandler):
    def _set_json(self, code=200):
        self.send_response(code)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()

    def do_GET(self):
        if self.path.startswith('/state'):
            self._set_json(200)
            self.wfile.write(json.dumps(state).encode())
        else:
            self.send_response(404); self.end_headers()

    def do_POST(self):
        if self.path == '/control/gpio':
            length = int(self.headers.get('Content-Length', '0'))
            body = self.rfile.read(length) if length else b''
            try:
                j = json.loads(body.decode())
                pin = int(j.get('pin', 0))
                value = int(j.get('value', 0))
                # update state
                if 0 <= pin < 32:
                    if value:
                        state['gpio_state'] |= (1 << pin)
                    else:
                        state['gpio_state'] &= ~(1 << pin)
                    print(f"[mock_core] gpio write pin={pin} value={value}")
                    self._set_json(200)
                    self.wfile.write(json.dumps({'status':'ok'}).encode())
                    return
            except Exception as e:
                print('bad post', e)
            self._set_json(400)
            self.wfile.write(json.dumps({'status':'error'}).encode())
        else:
            self.send_response(404); self.end_headers()

def tick_uptime():
    while True:
        time.sleep(1)
        state['uptime'] += 1

def run(server_class=HTTPServer, handler_class=Handler):
    server_address = ('127.0.0.1', 9000)
    httpd = server_class(server_address, handler_class)
    print('Mock core running on http://127.0.0.1:9000')
    t = threading.Thread(target=tick_uptime, daemon=True)
    t.start()
    httpd.serve_forever()

if __name__ == '__main__':
    run()
