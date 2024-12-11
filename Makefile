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
	docker build --tag wingb .

docker-run:
ifndef DOTENV_FILE
	$(error "DOTENV_FILE is not set, exiting")
endif
	docker ps | grep wingb | awk '{print $1}' | xargs -r -I {} docker stop {}
	docker run --env-file $$DOTENV_FILE --detach --publish 8010:8000 wingb:latest

deploy: check
	shuttle deploy

logs:
	shuttle logs

logs-error:
	shuttle logs | grep "ERROR" | tail -n 50
