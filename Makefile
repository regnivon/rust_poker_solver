

docker:
	docker build . -t gcr.io/pkr-solver/solver

push: docker
	docker push gcr.io/pkr-solver/solver