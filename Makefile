VENV ?= .venv

check:
	ruff format
	ruff check --fix --select I
	$(VENV)/bin/mypy main.py

precommit:
	cargo fmt
	cargo clippy

run: check
	$(VENV)bin/python main.py

watch:
	cargo watch --quiet  --watch static --watch templates --watch src --shell 'cargo run --bin wingb'

docker-build:
	docker build --tag wingb:latest .

docker-run:
ifndef DOTENV_FILE
	$(error "DOTENV_FILE is not set, exiting")
endif
	docker ps | grep wingb | awk '{print $$1}' | xargs -r -I {} docker stop {}
	docker run --name "wingb-$$(git rev-parse --short HEAD)-$$(date +%s)" --restart unless-stopped --env-file $$DOTENV_FILE --detach --publish 127.0.0.1:8010:8080 wingb:latest

deploy: check
	shuttle deploy

logs:
	shuttle logs

logs-error:
	shuttle logs | grep "ERROR" | tail -n 50
