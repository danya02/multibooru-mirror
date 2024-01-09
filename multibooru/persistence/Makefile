all: build redeploy

build:
	cd .. && docker buildx build --platform linux/arm64,linux/amd64 -f Dockerfile.persistence . --tag registry.danya02.ru/danya02/imageboards/persistence:latest --builder local --push

redeploy: delete deploy

deploy:
	kubectl apply -f deploy.yaml

delete:
	kubectl delete -f deploy.yaml ; exit 0

log:
	kubectl logs -n imageboards -f deployment/persistence-reader

initialize_builder:
	docker buildx create --bootstrap --name=local --driver=docker-container --platform=linux/arm64,linux/amd64

delete_builder:
	docker buildx rm local