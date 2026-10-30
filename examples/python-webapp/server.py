"""Zero-dependency HTTP example; usable via uv or Python directly."""
import argparse
import json
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--host', default=os.getenv('LAUNCHER_HOST', '127.0.0.1'))
parser.add_argument('--port', type=int, default=int(os.getenv('LAUNCHER_PORT', '3000')))
args = parser.parse_args()
Path(os.getenv('LAUNCHER_DATA_DIR', '.data')).mkdir(parents=True, exist_ok=True)

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/healthz':
            body, mime, code = json.dumps({'status': 'ready'}).encode(), 'application/json', 200
        elif self.path == '/':
            body, mime, code = b'<!doctype html><title>Flint Python example</title><h1>Python Web App is running</h1>', 'text/html; charset=utf-8', 200
        else:
            body, mime, code = b'Not found', 'text/plain', 404
        self.send_response(code)
        self.send_header('Content-Type', mime)
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = ThreadingHTTPServer((args.host, args.port), Handler)
print(f'Ready: http://{args.host}:{args.port}', flush=True)
try:
    server.serve_forever()
except KeyboardInterrupt:
    pass
finally:
    server.server_close()
