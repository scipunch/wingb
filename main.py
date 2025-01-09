import json
import logging
import os
import socket
import sys
import urllib
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Protocol, Self

import psycopg2

logging.basicConfig(level=os.environ.get("LOG_LEVEL", logging.INFO))
log = logging.getLogger(__name__)

SqlValue = str | int | bool

TEMPLATES_DIRECTORY = Path(os.getcwd()) / "templates/"
assert os.path.exists(TEMPLATES_DIRECTORY)


class SqlProvider(Protocol):
    def fetch(self, query: str) -> tuple[list[str], list[SqlValue]]: ...


def get_root(req: BaseHTTPRequestHandler) -> str:
    req.send_response(HTTPStatus.OK)
    req.send_header("Content-Type", "text/html")
    with open(TEMPLATES_DIRECTORY / "page" / "index.html") as f:
        template = f.read()
    return template


class HTMXRequestHandler(BaseHTTPRequestHandler):
    """Serves HTMX templates"""

    protocol_version = "HTTP/1.1"

    routes = {
        "GET": {"/": get_root},
    }

    def do_GET(self) -> None:
        content = self.send_head()
        if content is None:
            return

        self.wfile.write(content)

    def do_HEAD(self) -> None:
        self.send_head()

    def send_head(self) -> bytes | None:
        length = self.headers.get("content-length")
        try:
            nbytes = int(length or "0")
        except (TypeError, ValueError):
            nbytes = 0

        route_handler = self.routes["GET"].get(self.path)

        if route_handler is None:
            self.send_response(HTTPStatus.NOT_FOUND)
            return None

        content = route_handler(self)
        content_bytes = None

        if content:
            content_bytes = content.encode()
            self.send_header("Content-Length", str(len(content_bytes)))
        self.end_headers()

        return content_bytes


def main() -> None:
    infos = socket.getaddrinfo(
        None, 3000, type=socket.SOCK_STREAM, flags=socket.AI_PASSIVE
    )
    af, socktype, proto, canonname, sockaddr = next(iter(infos))
    HandlerClass = HTMXRequestHandler
    with ThreadingHTTPServer(sockaddr[:2], HandlerClass) as httpd:
        host, port = httpd.socket.getsockname()[:2]
        url_host = f"[{host}]" if ":" in host else host
        log.info(
            f"Serving HTMX on http://{url_host}:{port}/ with {HandlerClass.protocol_version}"
        )
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nKeyboard interrupt received, exiting.")
            sys.exit(0)


if __name__ == "__main__":
    main()
