import dataclasses
import inspect
import json
import logging
import os
import socket
import sys
import typing
import urllib
import urllib.parse
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import (
    Any,
    Callable,
    Literal,
    NamedTuple,
    NewType,
    Protocol,
    runtime_checkable,
)

import jinja2
import psycopg

logging.basicConfig(level=os.environ.get("LOG_LEVEL", logging.INFO))
log = logging.getLogger(__name__)

#  begin-region -- Domain

SqlValue = str | int | bool
HTTPMethod = Literal["GET", "POST"]
Htmx = NewType("Htmx", str)

TEMPLATES_DIRECTORY = Path(os.getcwd()) / "templates/"
assert os.path.exists(TEMPLATES_DIRECTORY)


@runtime_checkable
class SqlProvider(Protocol):
    def execute(self, query: str) -> list[NamedTuple]: ...


#  end-region   -- Domain

#  begin-region -- Context factory


def init_jinja2_env() -> jinja2.Environment:
    return jinja2.Environment(loader=jinja2.PackageLoader("wingb"))


def init_sql_provider() -> SqlProvider:
    return PostgreSqlProvider(psycopg.connect(os.environ["DATABASE_URL"]))


#  end-region   -- Context factory

#  begin-region -- Web


def get_root(
    req: BaseHTTPRequestHandler,
    jinja_env: jinja2.Environment,
) -> Htmx:
    req.send_response(HTTPStatus.OK)
    req.send_header("Content-Type", "text/html")
    template = jinja_env.get_template("page/index.html")
    return Htmx(template.render())


def post_generate(
    req: BaseHTTPRequestHandler,
    jinja_env: jinja2.Environment,
    sql_provider: SqlProvider,
) -> Htmx:
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

    sql_query = prompt

    try:
        table = sql_provider.execute(sql_query)
    except Exception as e:
        req.send_response(HTTPStatus.BAD_REQUEST)
        req.send_header("Content-Type", "text/html")
        return Htmx(f"SQL qeury execution failed with {e}")

    if not table:
        req.send_response(HTTPStatus.BAD_REQUEST)
        req.send_header("Content-Type", "text/html")
        return Htmx("Empty response from database")

    log.info(f"Got {table=}")
    req.send_response(HTTPStatus.OK)
    req.send_header("Content-Type", "text/html")
    return Htmx(
        jinja_env.get_template("component/sql-table.html").render(
            head=table[0]._fields,
            body=[tuple(row) for row in table],
            sql_query=sql_query,
        )
    )


class HTMXRequestHandler(BaseHTTPRequestHandler):
    """Serves HTMX templates"""

    protocol_version = "HTTP/1.1"

    routes: dict[HTTPMethod, dict[str, Callable[..., Any]]] = {
        "GET": {"/": get_root},
        "POST": {"/generate": post_generate},
    }
    context_factory = [init_sql_provider, init_jinja2_env]

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        self.context: list[Any] = []
        for factory in self.context_factory:
            self.context.append(factory())
        super().__init__(*args, **kwargs)

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

    def process_request(self, method: HTTPMethod) -> bytes | None:
        route_handler = self.routes[method].get(self.path)

        if route_handler is None:
            self.send_response(HTTPStatus.NOT_FOUND)
            return None

        typehints = typing.get_type_hints(route_handler)
        del typehints["return"]
        kwargs = {}
        for arg_name, arg_class in typehints.items():
            if arg_class == BaseHTTPRequestHandler:
                kwargs[arg_name] = self
                continue
            kwargs[arg_name] = next(d for d in self.context if isinstance(d, arg_class))

        content = route_handler(**kwargs)
        content_bytes = None

        if content:
            content_bytes = content.encode()
            self.send_header("Content-Length", str(len(content_bytes)))
        self.end_headers()

        return content_bytes


#  end-region   -- Web

#  begin-region -- Sql


@dataclasses.dataclass
class PostgreSqlProvider:
    conn: psycopg.Connection[Any]

    def execute(self, query: str) -> list[NamedTuple]:
        with self.conn.cursor(row_factory=psycopg.rows.namedtuple_row) as cur:
            rows = cur.execute(query).fetchall()

        return rows


#  end-region   -- Sql


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
