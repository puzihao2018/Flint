"""Small service implementing the Flint command convention; Python stdlib only."""
import argparse, json
from http.server import ThreadingHTTPServer,BaseHTTPRequestHandler
parser=argparse.ArgumentParser();parser.add_argument('--host',default='127.0.0.1');parser.add_argument('--port',required=True,type=int);args=parser.parse_args()
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        body=json.dumps({'app':'example.http-service','version':'0.1.0','status':'ready'}).encode()
        self.send_response(200 if self.path=='/healthz' else 404);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
server=ThreadingHTTPServer((args.host,args.port),Handler)
try:server.serve_forever()
except KeyboardInterrupt:pass
finally:server.server_close()
