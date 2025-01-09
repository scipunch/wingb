import json
import logging
import socket
import urllib
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Protocol

import psycopg2

logging.basicConfig(level=logging.INFO)
log = logging.getLogger(__name__)

SqlValue = str | int | bool


class SqlProvider(Protocol):
    def fetch(self, query: str) -> tuple[list[str], list[SqlValue]]: ...


class HTMXRequestHandler(BaseHTTPRequestHandler):
    """Serves HTMX templates"""

    protocol_version = "HTTP/1.1"

    def do_HEAD(self):
        length = self.headers.get("content-length")
        try:
            nbytes = int(length)
        except (TypeError, ValueError):
            nbytes = 0

        log.info(f"Request body {self.rfile.read(nbytes).decode()}")
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "text/plain")
        content = b"Hello there"
        self.send_header("Content-Length", len(content))
        self.end_headers()
        self.wfile.write(content)


def main():
    infos = socket.getaddrinfo(
        None, 3000, type=socket.SOCK_STREAM, flags=socket.AI_PASSIVE
    )
    af, socktype, proto, canonname, sockaddr = next(iter(infos))
    HandlerClass = HTMXRequestHandler
    with ThreadingHTTPServer(sockaddr, HandlerClass) as httpd:
        host, port = httpd.socket.getsockname()[:2]
        url_host = f"[{host}]" if ":" in host else host
        log.info(
            f"Serving HTMX on {host} port {port} with {HandlerClass.protocol_version} "
            f"(http://{url_host}:{port}/) ..."
        )
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nKeyboard interrupt received, exiting.")
            sys.exit(0)


if __name__ == "__main__":
    main()
