ifndef PROJECT
	PROJECT := aws-movies
endif

.PHONY: init build deploy destroy test

build:
	- mkdir target
	- rm target/*
	cd functions/movies-handler-graphql && make build
	cp functions/movies-handler-graphql/target/bootstrap.zip target/movies-handler-graphql.zip

init:
	cd terraform/aws && \
	terraform init \
		--backend-config="key=terraform/${PROJECT}/${REGION}/${STAGE}/terraform.tfstate" \
		--backend-config="region=${REGION}"

deploy:
	$(MAKE) init
	cd terraform/aws && \
	terraform apply \
		-var "project=${PROJECT}" \
		-var "stage=${STAGE}" \
		--auto-approve

destroy:
	$(MAKE) init
	cd terraform/aws && \
	terraform destroy \
		-var "project=${PROJECT}" \
		-var "stage=${STAGE}" \
		--auto-approve

test:
	python ./scripts/test.py
