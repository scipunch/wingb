check:
	cargo check

precommit:
	cargo fmt
	cargo clippy

run:
	cargo run

watch:
	cargo watch --quiet  --watch static --watch templates --watch src --shell 'shuttle run --external'

docker-build:
	docker build --tag wingb:latest .

docker-run:
ifndef DOTENV_FILE
	$(error "DOTENV_FILE is not set, exiting")
endif
	docker ps | grep wingb | awk '{print $$1}' | xargs -r -I {} docker stop {}
	docker run --name "wingb:$$(git rev-parse --short HEAD)-$$(date +%s)" --restart unless-stopped --env-file $$DOTENV_FILE --detach --publish 127.0.0.1:8010:8080 wingb:latest

deploy: check
	shuttle deploy

logs:
	shuttle logs

logs-error:
	shuttle logs | grep "ERROR" | tail -n 50
