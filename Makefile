.PHONY: build
build:
	cd functions/movies-handler-graphql && make build

.PHONY: deploy
deploy:
	cd terraform/aws && \
	terragrunt apply-all \
		--auto-approve \
		--terragrunt-non-interactive

.PHONY: destroy
destroy:
	cd terraform/aws && \
	terragrunt destroy-all \
		--auto-approve \
		--terragrunt-non-interactive

.PHONY: test
test:
	python ./scripts/test.py
