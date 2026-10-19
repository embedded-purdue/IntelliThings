#!/usr/bin/env python3
"""Work on the control panel without reflashing: panel_dev_server.py [--board 192.168.1.4] [--port 8765]

Serves the panel (esp32_c3_emulator/main/web/index.html) and the guide (GUIDE.md) straight
from this checkout, and forwards every other request (/api/...) to the board. Edit, reload
the browser at http://localhost:8765/, repeat. Standard library only.
"""

import argparse
import http.server
import pathlib
import urllib.error
import urllib.request

HERE = pathlib.Path(__file__).resolve().parent
PANEL = HERE.parent / "esp32_c3_emulator" / "main" / "web" / "index.html"
GUIDE = HERE.parent / "GUIDE.md"
BOARD = "http://192.168.1.4"
PASS_HEADERS = ("Content-Type", "Content-Encoding", "Content-Disposition", "Cache-Control")


class Handler(http.server.BaseHTTPRequestHandler):
    def _local(self, path, ctype, extra=None):
        body = path.read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        for k, v in (extra or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(body)

    def _proxy(self):
        length = int(self.headers.get("Content-Length") or 0)
        data = self.rfile.read(length) if length else None
        headers = {}
        if self.headers.get("Content-Type"):
            headers["Content-Type"] = self.headers["Content-Type"]
        req = urllib.request.Request(BOARD + self.path, data=data, headers=headers,
                                     method=self.command)
        try:
            with urllib.request.urlopen(req, timeout=20) as r:
                status, hdrs, body = r.status, r.headers, r.read()
        except urllib.error.HTTPError as e:
            status, hdrs, body = e.code, e.headers, e.read()
        except OSError as e:
            self.send_error(502, f"board unreachable: {e}")
            return
        self.send_response(status)
        for k in PASS_HEADERS:
            if hdrs.get(k):
                self.send_header(k, hdrs[k])
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/":
            self._local(PANEL, "text/html; charset=utf-8")
        elif path == "/guide.md":
            extra = {}
            if "download=" in self.path:
                extra["Content-Disposition"] = 'attachment; filename="IntelliThings-emulator-guide.md"'
            self._local(GUIDE, "text/markdown; charset=utf-8", extra)
        else:
            self._proxy()

    def do_POST(self):
        self._proxy()

    def log_message(self, fmt, *args):
        if not self.path.startswith("/api/poll"):
            super().log_message(fmt, *args)


def main():
    global BOARD
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--board", default="192.168.1.4")
    ap.add_argument("--port", type=int, default=8765)
    a = ap.parse_args()
    BOARD = "http://" + a.board
    print(f"Panel from {PANEL}\nAPI from {BOARD}\nOpen http://localhost:{a.port}/")
    http.server.ThreadingHTTPServer(("127.0.0.1", a.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
