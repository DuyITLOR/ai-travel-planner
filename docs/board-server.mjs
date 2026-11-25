#!/usr/bin/env node
// Server nhỏ cho task board: mở docs/tasks.html ở http://127.0.0.1 và lưu thẳng vào file,
// không phải chọn file hay cấp quyền trong Chrome. Không cần thư viện ngoài.
//
//   pnpm board                 mở board và tự mở trình duyệt
//   PORT=5000 pnpm board       đổi cổng
//   BOARD_NO_OPEN=1 pnpm board không tự mở trình duyệt
//
// Chỉ nghe trên 127.0.0.1; chỉ đọc và ghi đúng một file board.
import { createServer } from 'node:http';
import { readFile, writeFile, rename } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import path from 'node:path';

const FILE = path.resolve(process.env.BOARD_FILE || fileURLToPath(new URL('./tasks.html', import.meta.url)));
const HOST = '127.0.0.1';
const PORT = Number(process.env.PORT || 4777);
const URL_ = `http://${HOST}:${PORT}/`;
const DATA_OPEN = '<script type="application/json" id="tasks-data">';
const MAX_BYTES = 5 * 1024 * 1024;

// Chặn trang web lạ gọi vào server này (DNS rebinding, form từ site khác).
const okHost = (h) => h === `${HOST}:${PORT}` || h === `localhost:${PORT}`;
function okOrigin(origin) {
  if (!origin) return true;
  try {
    return okHost(new URL(origin).host);
  } catch {
    return false;
  }
}
function isBoard(src) {
  const i = src.lastIndexOf(DATA_OPEN);
  return i >= 0 && src.indexOf('</script>', i) > 0;
}
function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let size = 0;
    req.on('data', (c) => {
      size += c.length;
      if (size > MAX_BYTES) {
        reject(Object.assign(new Error('File quá lớn'), { status: 413 }));
        req.destroy();
      } else chunks.push(c);
    });
    req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    req.on('error', reject);
  });
}

const server = createServer(async (req, res) => {
  const send = (status, body = '', type = 'text/plain; charset=utf-8') => {
    res.writeHead(status, { 'content-type': type, 'cache-control': 'no-store', 'x-content-type-options': 'nosniff' });
    res.end(body);
  };
  if (!okHost(req.headers.host || '')) return send(403, 'Host không hợp lệ');
  const { pathname } = new URL(req.url, URL_);
  try {
    if (req.method === 'GET' && (pathname === '/' || pathname === '/tasks.html')) {
      return send(200, await readFile(FILE, 'utf8'), 'text/html; charset=utf-8');
    }
    if (pathname === '/__board/source') {
      if (req.method === 'GET') return send(200, await readFile(FILE, 'utf8'));
      if (req.method === 'PUT') {
        if (!okOrigin(req.headers.origin)) return send(403, 'Origin không hợp lệ');
        const body = await readBody(req);
        if (!isBoard(body)) return send(400, 'Nội dung gửi lên không phải board (thiếu khối dữ liệu task).');
        if (/^(<{7}|>{7})( |$)/m.test(body)) return send(400, 'Nội dung có dấu conflict git.');
        const tmp = FILE + '.tmp';
        await writeFile(tmp, body, 'utf8');
        await rename(tmp, FILE); // ghi file tạm rồi đổi tên: không bao giờ để lại file ghi dở
        return send(204);
      }
      return send(405, 'Không hỗ trợ');
    }
    return send(404, 'Không có');
  } catch (e) {
    return send(e.status || 500, e.message);
  }
});

server.on('error', (e) => {
  if (e.code === 'EADDRINUSE') {
    console.error(`Cổng ${PORT} đang bận. Có thể board đã chạy rồi: mở ${URL_}\nHoặc đổi cổng: PORT=5000 pnpm board`);
  } else {
    console.error(e.message);
  }
  process.exit(1);
});

server.listen(PORT, HOST, () => {
  console.log(`Task board: ${URL_}`);
  console.log(`Lưu vào: ${path.relative(process.cwd(), FILE) || FILE}`);
  console.log('Ctrl+C để tắt.');
  if (process.env.BOARD_NO_OPEN) return;
  const [cmd, args] =
    process.platform === 'darwin'
      ? ['open', [URL_]]
      : process.platform === 'win32'
        ? ['cmd', ['/c', 'start', '', URL_]]
        : ['xdg-open', [URL_]];
  spawn(cmd, args, { stdio: 'ignore', detached: true })
    .on('error', () => {})
    .unref();
});
