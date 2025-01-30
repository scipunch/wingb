VENV ?= .venv
DOCKER_IMAGE = wingb

pre-commit: format lint

format:
	uv run ruff format

lint: format
	uv run ruff check --fix
	uv run mypy .

run: pre-commit
	uv run python wingb.py

docker-build:
	docker build --tag $(DOCKER_IMAGE):latest .

# Creates container with name image-commit-timestamp format
docker-run:
ifndef DOTENV_FILE
	$(error "DOTENV_FILE is not set, exiting")
endif
ifndef WINGB_ADDITIONAL_CONTEXT
	$(error "WINGB_ADDITIONAL_CONTEXT is not set, exiting")
endif
	docker ps | grep $(DOCKER_IMAGE) | awk '{print $$1}' | xargs -r -I {} docker stop {}
	docker run --name "$(DOCKER_IMAGE)-$$(git rev-parse --short HEAD)-$$(date +%s)" --restart unless-stopped --env-file $$DOTENV_FILE --detach --publish 127.0.0.1:8010:3000 -e $$WINGB_ADDITIONAL_CONTEXT -v $$WINGB_ADDITIONAL_CONTEXT:$$WINGB_ADDITIONAL_CONTEXT:ro  $(DOCKER_IMAGE):latest
