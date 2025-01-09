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
import urllib.request
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import (
    Any,
    Callable,
    Literal,
    NamedTuple,
    NewType,
    Optional,
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

TEMPLATES_DIRECTORY = os.path.join(os.getcwd(), "templates/")
assert os.path.exists(TEMPLATES_DIRECTORY)

# TODO: Make a dependency instead of global
ADDITIONAL_CONTEXT_PATH = "additional_context.txt"
ADDITIONAL_CONTEXT = None
if os.path.exists(ADDITIONAL_CONTEXT_PATH):
    with open(ADDITIONAL_CONTEXT_PATH) as f:
        ADDITIONAL_CONTEXT = f.read()

if ADDITIONAL_CONTEXT is None:
    log.warn("Empty additional context")


@runtime_checkable
class SqlProvider(Protocol):
    def execute(self, query: str) -> list[NamedTuple]: ...
    def read_database_schema(self) -> list[str]: ...


@runtime_checkable
class LLMProvider(Protocol):
    def convert_to_sql(
        self, database_schema: str, additional_context: Optional[str], prompt: str
    ) -> str: ...


#  end-region   -- Domain

#  begin-region -- Context factory


def init_jinja2_env() -> jinja2.Environment:
    return jinja2.Environment(loader=jinja2.PackageLoader("wingb", TEMPLATES_DIRECTORY))


def init_sql_provider() -> SqlProvider:
    return PostgreSqlProvider(psycopg.connect(os.environ["DATABASE_URL"]))


def init_llm_provider() -> LLMProvider:
    return OpenAILLMProvider(os.environ["OPENAI_API_KEY"])


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
    llm_provider: LLMProvider,
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

    database_schema = sql_provider.read_database_schema()

    try:
        sql_query = llm_provider.convert_to_sql(
            "\n".join(database_schema),
            ADDITIONAL_CONTEXT,
            prompt,
        )
        log.info(f"{sql_query=}")
    except urllib.error.HTTPError as e:
        msg = f"Failed to send request to LLM with {e=} and body={e.read().decode()}"
        log.error(msg)
        req.send_response(HTTPStatus.INTERNAL_SERVER_ERROR)
        req.send_header("Content-Type", "text/html")
        return Htmx(msg)

    try:
        table = sql_provider.execute(sql_query)
    except Exception as e:
        msg = f"SQL qeury execution failed with {type(e)}: {e}"
        log.error(msg)
        req.send_response(HTTPStatus.BAD_REQUEST)
        req.send_header("Content-Type", "text/html")
        return Htmx(msg)

    if not table:
        req.send_response(HTTPStatus.BAD_REQUEST)
        req.send_header("Content-Type", "text/html")
        return Htmx("Empty response from database")

    log.info(f"Got {len(table)} rows")
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
    context_factory = [init_sql_provider, init_jinja2_env, init_llm_provider]

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

    def _execute(self, query: str) -> list[NamedTuple]:
        with self.conn.cursor(row_factory=psycopg.rows.namedtuple_row) as cur:
            return cur.execute(query).fetchall()

    def execute(self, query: str) -> list[NamedTuple]:
        log.info(f"Execting {query=}")
        return self._execute(query)

    def read_database_schema(self) -> list[str]:
        sql = """
WITH table_columns AS (
    SELECT 
        table_schema,
        table_name,
        column_name,
        CASE 
            WHEN data_type = 'character varying' THEN 'VARCHAR(' || character_maximum_length || ')'
            WHEN data_type = 'character' THEN 'CHAR(' || character_maximum_length || ')'
            WHEN data_type = 'numeric' THEN 'NUMERIC(' || numeric_precision || ',' || numeric_scale || ')'
            ELSE data_type
        END AS column_definition,
        CASE 
            WHEN is_nullable = 'NO' THEN ' NOT NULL'
            ELSE ''
        END AS null_constraint
    FROM 
        information_schema.columns
    WHERE 
        table_schema NOT IN ('pg_catalog', 'information_schema')
),
foreign_keys AS (
    SELECT 
        kcu.table_schema,
        kcu.table_name,
        kcu.column_name,
        ccu.table_schema AS foreign_table_schema,
        ccu.table_name AS foreign_table_name,
        ccu.column_name AS foreign_column_name
    FROM 
        information_schema.key_column_usage kcu
    JOIN 
        information_schema.referential_constraints rc ON kcu.constraint_name = rc.constraint_name
    JOIN 
        information_schema.constraint_column_usage ccu ON ccu.constraint_name = rc.unique_constraint_name
)
SELECT 
    'CREATE TABLE ' || tc.table_schema || '.' || tc.table_name || ' (' || 
    string_agg(tc.column_name || ' ' || tc.column_definition || tc.null_constraint, ', ') || 
    CASE 
        WHEN COUNT(fk.foreign_table_name) > 0 THEN ', ' || string_agg('FOREIGN KEY (' || fk.column_name || ') REFERENCES ' || fk.foreign_table_schema || '.' || fk.foreign_table_name || '(' || fk.foreign_column_name || ')', ', ')
        ELSE ''
    END || 
    ');' AS create_table_statement
FROM 
    table_columns tc
LEFT JOIN 
    foreign_keys fk ON tc.table_schema = fk.table_schema AND tc.table_name = fk.table_name AND tc.column_name = fk.column_name
GROUP BY 
    tc.table_schema, tc.table_name
ORDER BY 
    tc.table_schema, tc.table_name;"""

        rows = self._execute(sql)
        return [getattr(row, "create_table_statement") for row in rows]


#  end-region   -- Sql

#  begin-region -- LLM Provider


@dataclasses.dataclass
class OpenAILLMProvider:
    openai_api_token: str

    def convert_to_sql(
        self, database_schema: str, additional_context: Optional[str], prompt: str
    ) -> str:
        log.info(
            f"Converting {prompt=} with {additional_context and len(additional_context)} context"
        )
        messages = [
            {
                "role": "developer",
                "content": """You are a helpful assistant specialising in data analysis
                              in a PostgreSQL database. Answer the questions by providing
                              raw SQL code that is compatible with the PostgreSQL.""",
            },
            {
                "role": "user",
                "content": f"Here is a database schema:\n```sql\n{database_schema}\n```",
            },
            {
                "role": "user",
                "content": f"And description for some tables:\n{additional_context}",
            },
            {"role": "user", "content": prompt},
        ]
        response_format = {
            "type": "json_schema",
            "json_schema": {
                "name": "sql_query_schema",
                "schema": {
                    "type": "object",
                    "properties": {
                        "sql_query": {
                            "description": "Result SQL query that will be passted to `psycopg.Donnection.execute` method",
                            "type": "string",
                        }
                    },
                },
            },
        }
        model = "gpt-4o-mini"
        data = json.dumps(
            {"messages": messages, "model": model, "response_format": response_format}
        ).encode()
        req = urllib.request.Request(
            "https://api.openai.com/v1/chat/completions",
            data=data,
            method="POST",
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.openai_api_token}",
            },
        )
        log.info(f"Sending {req=}")
        with urllib.request.urlopen(req) as resp:
            resp_body = json.load(resp)

        log.debug(f"ChatGPT {resp_body=}")
        sql_query = json.loads(resp_body["choices"][0]["message"]["content"])[
            "sql_query"
        ]
        assert isinstance(sql_query, str)
        return sql_query


#  end-region   -- LLM Provider


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
