.PHONY: build
build:
	cd functions/movies-handler-graphql && make build

.PHONY: deploy
deploy:
	cd terraform/aws && \
	terragrunt apply-all \
		--terragrunt-non-interactive

.PHONY: destroy
destroy:
	cd terraform/aws && \
	terragrunt destroy-all \
		--terragrunt-non-interactive

.PHONY: test
test:
	cd functions/movies-handler-graphql && make test

.PHONY: test-aws
test-aws:
	python ./scripts/test.py
