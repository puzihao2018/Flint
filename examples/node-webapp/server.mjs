// No dependencies. Flint supplies these variables; ordinary node startup also works.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
const port = Number(process.env.LAUNCHER_PORT ?? process.env.PORT ?? 3000);
const host = process.env.LAUNCHER_HOST ?? '127.0.0.1';
const data = process.env.LAUNCHER_DATA_DIR ?? path.resolve('.data');
fs.mkdirSync(data, { recursive: true });
const server = http.createServer((req, res) => {
  if (req.url === '/healthz') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    return res.end(JSON.stringify({ status: 'ready' }));
  }
  if (req.url === '/') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return res.end('<!doctype html><title>Flint Node example</title><h1>Node Web App is running</h1><p>Start, change port, and stop this service in Flint.</p>');
  }
  res.writeHead(404).end();
});
server.on('error', error => { console.error(error); process.exit(1); });
server.listen(port, host, () => console.log(`Ready: http://${host}:${port}`));
function stop() {
  server.close(() => process.exit(0));
  setTimeout(() => process.exit(1), 4000).unref();
}
process.on('SIGINT', stop);
process.on('SIGTERM', stop);
