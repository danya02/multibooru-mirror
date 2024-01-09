all: build redeploy

build:
	cd ../../.. && docker buildx build --platform linux/arm64,linux/amd64 -f Dockerfile.danbooru-comment-writer . --tag registry.danya02.ru/danya02/imageboards/danbooru-comment-writer:latest --builder local --push

redeploy:
	kubectl delete -f deploy.yaml ; exit 0
	sleep 5
	kubectl apply -f deploy.yaml


deploy:
	kubectl apply -f deploy.yaml

delete:
	kubectl delete -f deploy.yaml ; exit 0

log:
	kubectl logs -n imageboards -f deployment/danbooru-comment-writer -f


initialize_builder:
	docker buildx create --bootstrap --name=local --driver=docker-container --platform=linux/arm64,linux/amd64

delete_builder:
	docker buildx rm local