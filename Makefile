NAME ?= ictsc-discord-bot

IMAGE_TAG ?= latest
IMAGE ?= ghcr.io/ictsc/ictsc-discord-bot:$(IMAGE_TAG)

DOCKER_ARGS ?= --name $(NAME) -v "$(shell pwd)/bot.yaml:/bot.yaml"

all:

.PHONY: build
build:
	DOCKER_BUILDKIT=1 docker build -t $(IMAGE) .

.PHONY: start
start:
	DOCKER_BUILDKIT=1 docker run -d $(DOCKER_ARGS) $(IMAGE) -f /bot.yaml start

.PHONY: start
stop:
	DOCKER_BUILDKIT=1 docker rm -f $(NAME)
