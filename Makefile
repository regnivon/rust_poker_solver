docker:
	docker build . -t gcr.io/poker-sims/solver

push: docker
	docker push gcr.io/poker-sims/solver
