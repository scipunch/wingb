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
	docker build --cpu-quota 100000 --tag wingb .

deploy: check
	shuttle deploy

logs:
	shuttle logs

logs-error:
	shuttle logs | grep "ERROR" | tail -n 50
