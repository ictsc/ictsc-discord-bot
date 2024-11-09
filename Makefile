NAME ?= ictsc-discord-bot

IMAGE_TAG ?= latest
IMAGE ?= ghcr.io/ictsc/ictsc-discord-bot:$(IMAGE_TAG)

DOCKER_ARGS ?= -v "$(shell pwd)/bot.yaml:/bot.yaml" --net host --env RUST_LOG=info,bot=trace

all:

.PHONY: fmt
fmt:
	cargo +nightly fmt

.PHONY: build
build:
	DOCKER_BUILDKIT=1 docker build -t $(IMAGE) .

.PHONY: start
start:
	docker run -d --name $(NAME) $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml start

.PHONY: stop
stop:
	docker rm -f $(NAME)

.PHONY: sync
sync: bot.yaml
	docker run -it $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml $@

.PHONY: flush
flush:
	docker run -it $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml delete-channels
	docker run -it $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml delete-roles
	docker run -it $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml delete-commands

.PHONY: logs
logs:
	docker logs -f $(NAME)
