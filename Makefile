COMPOSE := docker-compose -f docker-compose.dev.yml

.PHONY: help up down restart ps logs logs-api logs-web logs-db build build-api build-web \
	test test-api test-web lint lint-web typecheck typecheck-web shell-api shell-web \
	hooks-install fmt

help:
	@printf "%s\n" \
		"Garage360 workflow commands" \
		"" \
		"Lifecycle:" \
		"  make up            Start the local Docker development stack" \
		"  make down          Stop the local Docker development stack" \
		"  make restart       Restart the local Docker development stack" \
		"  make ps            Show running services" \
		"" \
		"Logs:" \
		"  make logs          Follow all service logs" \
		"  make logs-api      Follow API logs" \
		"  make logs-web      Follow web logs" \
		"  make logs-db       Follow control DB logs" \
		"" \
		"Build:" \
		"  make build         Build API and web images/services" \
		"  make build-api     Build the API image" \
		"  make build-web     Build the web service image/deps layer" \
		"" \
		"Quality:" \
		"  make test          Run API and web tests in Docker" \
		"  make test-api      Run API tests in Docker" \
		"  make test-web      Run web tests in Docker" \
		"  make lint          Run frontend lint in Docker" \
		"  make typecheck     Run frontend typecheck in Docker" \
		"" \
		"Shells:" \
		"  make shell-api     Open a shell in the API container" \
		"  make shell-web     Open a shell in the web container" \
		"" \
		"Git workflow:" \
		"  make hooks-install Configure git to use .githooks/"

up:
	$(COMPOSE) up -d

down:
	$(COMPOSE) down

restart: down up

ps:
	$(COMPOSE) ps

logs:
	$(COMPOSE) logs -f

logs-api:
	$(COMPOSE) logs -f api

logs-web:
	$(COMPOSE) logs -f web

logs-db:
	$(COMPOSE) logs -f control-db

build: build-api build-web

build-api:
	$(COMPOSE) build api

build-web:
	$(COMPOSE) build web

test: test-api test-web

test-api:
	$(COMPOSE) exec api cargo test

test-web:
	$(COMPOSE) exec web npm test

lint: lint-web

lint-web:
	$(COMPOSE) exec web npm run lint

typecheck: typecheck-web

typecheck-web:
	$(COMPOSE) exec web npm run typecheck

fmt:
	$(COMPOSE) exec api cargo fmt
	$(COMPOSE) exec web npm run lint -- --fix

shell-api:
	$(COMPOSE) exec api sh

shell-web:
	$(COMPOSE) exec web sh

hooks-install:
	git config core.hooksPath .githooks
	@echo "Configured git hooks from .githooks/"
