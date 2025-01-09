FROM python:3.13

RUN pip install jinja2 psycopg

WORKDIR app
COPY wingb.py .
COPY templates ./templates

CMD python wingb.py
