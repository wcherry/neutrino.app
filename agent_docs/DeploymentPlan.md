Below is a clean production deployment pattern for Rust microservices using:
	•	GitHub
	•	GitHub Container Registry
	•	Docker
	•	Docker Compose
	•	Rust

This pattern is widely used because:
	•	builds happen once in CI
	•	servers only pull images
	•	deploys are atomic
	•	failed releases automatically rollback

⸻

Architecture

Developer push
      │
      ▼
GitHub Actions
  build Rust images
  push → GHCR
      │
      ▼
Server deployment script
  docker compose pull
  docker compose up -d
      │
      ▼
Health check verification
      │
   ┌──┴────────────┐
   │               │
Healthy        Unhealthy
   │               │
   ▼               ▼
Success       Rollback


⸻

1. Multi-Stage Rust Dockerfile

This keeps runtime images small and secure.

Dockerfile

# -------- Build stage --------
FROM rust:1.93 as builder

WORKDIR /app

# cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# build actual code
COPY src ./src
RUN cargo build --release

# -------- Runtime stage --------
FROM debian:bookworm-slim

RUN useradd -m appuser

WORKDIR /app

COPY --from=builder /app/target/release/myservice /usr/local/bin/myservice

USER appuser

EXPOSE 8080

CMD ["myservice"]

Typical result:

build image: ~1.8GB
runtime image: ~50–80MB


⸻

2. Docker Compose (Production)

docker-compose.yml

version: "3.9"

services:

  api:
    image: ghcr.io/myorg/api:latest
    restart: always

    ports:
      - "8080:8080"

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 3s
      retries: 5

  worker:
    image: ghcr.io/myorg/worker:latest
    restart: always

    healthcheck:
      test: ["CMD", "pgrep", "worker"]
      interval: 10s
      retries: 5

Important: health checks drive rollback logic.

⸻

3. GitHub Actions CI Pipeline

.github/workflows/build.yml

name: Build and Publish

on:
  push:
    branches: [ main ]

permissions:
  contents: read
  packages: write

jobs:

  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Login to GHCR
        run: echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u ${{ github.actor }} --password-stdin

      - name: Build image
        run: |
          docker build -t ghcr.io/${{ github.repository }}/api:latest .

      - name: Push image
        run: |
          docker push ghcr.io/${{ github.repository }}/api:latest

This automatically pushes to:

ghcr.io/org/repo/api:latest


⸻

4. Server Deployment Script

This script is what makes the system robust.

deploy.sh

#!/bin/bash

set -e

STACK_DIR="/srv/myproject"

cd $STACK_DIR

echo "Saving current image versions..."
docker compose images > previous-images.txt

echo "Pulling new images..."
docker compose pull

echo "Starting new containers..."
docker compose up -d

echo "Waiting for health checks..."

sleep 20

FAILED=$(docker ps --filter "health=unhealthy" --format "{{.Names}}")

if [ -n "$FAILED" ]; then
  echo "Health check failed. Rolling back..."

  while read line; do
    IMAGE=$(echo $line | awk '{print $2}')
    SERVICE=$(echo $line | awk '{print $1}')

    docker compose stop $SERVICE
    docker run -d $IMAGE
  done < previous-images.txt

  exit 1
fi

echo "Deployment successful"


⸻

5. Deployment Trigger

GitHub can trigger deployment via SSH.

Example:

.github/workflows/deploy.yml

name: Deploy

on:
  workflow_run:
    workflows: ["Build and Publish"]
    types:
      - completed

jobs:
  deploy:

    runs-on: ubuntu-latest

    steps:
      - name: SSH Deploy
        uses: appleboy/ssh-action@v1
        with:
          host: ${{ secrets.SERVER_HOST }}
          username: deploy
          key: ${{ secrets.SERVER_KEY }}
          script: |
            /srv/myproject/deploy.sh


⸻

6. Server Setup

Server only needs:

docker
docker compose
git

Project layout:

/srv/myproject

docker-compose.yml
deploy.sh
.env


⸻

7. Image Authentication

Login once on the server:

docker login ghcr.io

Use a GitHub PAT with:

read:packages

This allows pulling private images from GitHub Container Registry.

⸻

8. Why This Is Much More Reliable

Compared to rebuilding on the server:

Feature	Old Method	This Method
Build location	server	CI
Deploy speed	slow	fast
Reproducibility	inconsistent	deterministic
Rollback	manual	automatic
Resource usage	high	low

Deploys typically take:

3–8 seconds

instead of minutes.

⸻

9. Optional Production Improvements

Highly recommended additions:

Immutable image tags

Instead of latest:

ghcr.io/org/api:git-sha


⸻

Automatic cleanup

docker image prune -af


⸻

Service dependency ordering

depends_on:
  postgres:
    condition: service_healthy


⸻

10. Typical Microservice Layout

repo/

services/
  api/
  worker/
  auth/

docker/
  api.Dockerfile
  worker.Dockerfile

docker-compose.yml

.github/workflows/
  build.yml
  deploy.yml


⸻

💡 If you’d like, I can also show a much more advanced version of this setup used by high-scale Rust teams that adds:
	•	zero-downtime deploys
	•	blue/green containers
	•	instant rollback (<1s)
	•	layer-cached Rust builds (~15s CI)

It’s a huge improvement when running 5–20 microservices.




docker build --build-arg SERVICE=auth
  -t ghcr.io/org/api:latest \
  .


  docker build \
  --build-arg SERVICE=api \
  -t ghcr.io/org/api:latest \
  .