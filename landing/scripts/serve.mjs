#!/usr/bin/env node
// Local preview server: builds the site with BASE_PATH=/ and serves landing/dist.
//   npm run preview            (PORT=4173 HOST=127.0.0.1 by default)
//   node scripts/serve.mjs --no-build
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { build } from './build.mjs';
import { LANDING_DIR } from './lib/repo.mjs';

const MIME = {
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8', '.svg': 'image/svg+xml', '.xml': 'application/xml', '.txt': 'text/plain; charset=utf-8',
};
const outDir = path.join(LANDING_DIR, 'dist');
if (!process.argv.includes('--no-build')) build({ base: '/', outDir });

const host = process.env.HOST || '127.0.0.1';
const port = Number(process.env.PORT || 4173);
const server = http.createServer((req, res) => {
  let rel = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  let file = path.join(outDir, rel);
  if (!file.startsWith(outDir)) { res.writeHead(403).end('Forbidden'); return; }
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, 'index.html');
  if (!fs.existsSync(file)) {
    res.writeHead(404, { 'Content-Type': MIME['.html'] }).end(fs.readFileSync(path.join(outDir, '404.html')));
    return;
  }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] || 'application/octet-stream', 'Cache-Control': 'no-store' });
  fs.createReadStream(file).pipe(res);
});
server.listen(port, host, () => console.log(`Preview: http://${host}:${port}/  (Ctrl+C to stop)`));
