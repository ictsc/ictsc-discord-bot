VERSION := $(shell git rev-parse --short HEAD)

all:

precommit: fmt fix

fmt:
	cargo fmt

fix:
	cargo fix --allow-dirty --allow-staged

build:
	DOCKER_BUILDKIT=1 docker build -t ictsc/ictsc-kana:${VERSION} .

run:
	docker run --name kana -itd -v $(shell pwd)/bot.yaml:/bot.yaml:ro -e RUST_LOG=bot=debug,info ictsc/ictsc-kana:${VERSION} -f /bot.yaml start

rm:
	docker rm -f kana

.PHONY: precommit fmt fix build run rm
