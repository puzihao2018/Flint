"""Local lifecycle/MCP fixture, no third-party dependencies."""
import argparse, json, os, signal, subprocess, sys
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
p=argparse.ArgumentParser();p.add_argument('--port',type=int,required=True);p.add_argument('--mode',default='http');a=p.parse_args()
if a.mode=='crash': sys.exit(23)
if a.mode=='address-in-use':
    print('Error: listen EADDRINUSE: address already in use', file=sys.stderr, flush=True)
    sys.exit(1)
if a.mode=='tree':
    child=subprocess.Popen([sys.executable,__file__,'--port',str(a.port),'--mode','stubborn'])
    open('child.pid','w').write(str(child.pid))
    child.wait();sys.exit()
if a.mode=='stubborn': signal.signal(signal.SIGINT,signal.SIG_IGN)
class Handler(BaseHTTPRequestHandler):
    def send(self,code,body=b'',mime='application/json',session=False):
        self.send_response(code);self.send_header('Content-Type',mime);self.send_header('Content-Length',str(len(body)))
        if session:self.send_header('Mcp-Session-Id','test-session')
        self.end_headers();self.wfile.write(body)
    def do_GET(self):self.send(500 if a.mode=='bad-health' else 200,b'{"ok":true}')
    def do_POST(self):
        msg=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        if msg.get('method')=='initialize':
            value=json.dumps({'jsonrpc':'2.0','id':1,'result':{'protocolVersion':'2025-03-26','capabilities':{},'serverInfo':{'name':'fixture','version':'1'}}}).encode()
            if a.mode=='mcp-sse':
                value=b': keepalive\n\ndata: {"jsonrpc":"2.0","method":"notifications/message","params":{}}\n\nevent: message\ndata: '+value+b'\n\n'
            self.send(200,value,'text/event-stream' if a.mode=='mcp-sse' else 'application/json',True)
        elif msg.get('method')=='notifications/initialized':
            assert self.headers.get('Mcp-Session-Id')=='test-session'
            open('initialized','w').write('yes');self.send(202)
        else:self.send(400)
    def do_DELETE(self):
        assert self.headers.get('Mcp-Session-Id')=='test-session'
        open('deleted','w').write('yes');self.send(200)
ThreadingHTTPServer(('127.0.0.1',a.port),Handler).serve_forever()
