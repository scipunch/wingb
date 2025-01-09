import json
import logging
import os
import socket
import sys
import urllib
import urllib.parse
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Literal, NewType, Protocol

import jinja2
import psycopg2

logging.basicConfig(level=os.environ.get("LOG_LEVEL", logging.INFO))
log = logging.getLogger(__name__)

jinja_env = jinja2.Environment(loader=jinja2.PackageLoader("wingb"))

#  begin-region -- Domain

SqlValue = str | int | bool
Htmx = NewType("Htmx", str)

TEMPLATES_DIRECTORY = Path(os.getcwd()) / "templates/"
assert os.path.exists(TEMPLATES_DIRECTORY)


class SqlProvider(Protocol):
    def fetch(self, query: str) -> tuple[list[str], list[SqlValue]]: ...


#  end-region   -- Domain

#  begin-region -- Web


def get_root(req: BaseHTTPRequestHandler) -> Htmx:
    req.send_response(HTTPStatus.OK)
    req.send_header("Content-Type", "text/html")
    template = jinja_env.get_template("page/index.html")
    return Htmx(template.render())


def post_generate(req: BaseHTTPRequestHandler) -> Htmx:
    length = req.headers.get("content-length")
    try:
        nbytes = int(length or "0")
    except (TypeError, ValueError):
        nbytes = 0
    body = urllib.parse.parse_qs(req.rfile.read(nbytes).decode())

    prompt = "\n".join(body.get("prompt", []))
    if not prompt:
        req.send_response(HTTPStatus.BAD_REQUEST)
        req.send_header("Content-Type", "text/html")
        return Htmx("Form 'prompt' field missing")

    req.send_response(HTTPStatus.OK)
    req.send_header("Content-Type", "text/html")
    template = jinja_env.get_template("component/sql-table.html")
    return Htmx(
        template.render(
            head=["foo", "bar"],
            body=[["foo1", "bar1"], ["foo2", "bar2"]],
            sql_query="Some sql query",
        )
    )


class HTMXRequestHandler(BaseHTTPRequestHandler):
    """Serves HTMX templates"""

    protocol_version = "HTTP/1.1"

    routes = {"GET": {"/": get_root}, "POST": {"/generate": post_generate}}

    def do_POST(self) -> None:
        content = self.process_request("POST")
        if content is None:
            return

        self.wfile.write(content)

    def do_GET(self) -> None:
        content = self.process_request("GET")
        if content is None:
            return

        self.wfile.write(content)

    def do_HEAD(self) -> None:
        self.process_request("GET")

    def process_request(self, method: Literal["GET", "POST"]) -> bytes | None:
        route_handler = self.routes[method].get(self.path)

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


#  end-region   -- Web


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
