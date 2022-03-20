docker:
	docker build . --tag gcr.io/poker-sims/solver:0.0.3

push: docker
	docker push gcr.io/poker-sims/solver:0.0.3
