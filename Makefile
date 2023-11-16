NAME ?= ictsc-discord-bot

IMAGE_TAG ?= latest
IMAGE ?= ghcr.io/ictsc/ictsc-discord-bot:$(IMAGE_TAG)

DOCKER_ARGS ?= -v "$(shell pwd)/bot.yaml:/bot.yaml" --net host --env RUST_LOG=info,bot=debug

all:

.PHONY: build
build:
	DOCKER_BUILDKIT=1 docker build -t $(IMAGE) .

.PHONY: start
start:
	docker run -d --name $(NAME) $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml start

.PHONY: sync-channels sync-roles
sync-channels sync-roles:
	docker run -it $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml $@

.PHONY: stop
stop:
	docker rm -f $(NAME)

.PHONY: logs
logs:
	docker logs -f $(NAME)

