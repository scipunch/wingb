check:
	cargo check

run:
	cargo run

watch:
	cargo watch --watch static --watch templates --watch src --exec run


deploy: check
	shuttle deploy
