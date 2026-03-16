# ---------- Build Stage ----------
FROM rust:1.93 AS builder

WORKDIR /app

# Copy workspace manifests first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

COPY shared/ shared/
COPY auth/Cargo.toml auth/
COPY drive/Cargo.toml drive/
COPY docs/Cargo.toml docs/
COPY sheets/Cargo.toml sheets/
COPY slides/Cargo.toml slides/
COPY worker/Cargo.toml worker/

# Create dummy source files to build dependency cache
RUN mkdir -p auth/src \
 && mkdir -p drive/src \
 && mkdir -p docs/src \
 && mkdir -p sheets/src \
 && mkdir -p slides/src \
 && mkdir -p worker/src \
 && echo "fn main(){}" > auth/src/main.rs \
 && echo "fn main(){}" > drive/src/main.rs \
 && echo "fn main(){}" > docs/src/main.rs \
 && echo "fn main(){}" > sheets/src/main.rs \
 && echo "fn main(){}" > slides/src/main.rs \
 && echo "fn main(){}" > worker/src/main.rs 

RUN cargo build --release --workspace

# Remove dummy sources
RUN rm -rf shared auth drive docs sheets worker

# Copy real source
COPY shared shared
COPY auth auth
COPY drive drive
COPY docs docs
COPY sheets sheets
COPY slides slides
COPY worker worker

# Build selected service
ARG SERVICE

RUN cargo build --release -p $SERVICE

# ---------- Runtime Stage ----------
FROM debian:bookworm-slim

RUN useradd -m appuser
WORKDIR /app
RUN mkdir -p /usr/local/data /usr/local/logs \
    && chown -R appuser:appuser /usr/local/data /usr/local/logs

RUN apt-get update \
    && apt-get install -y openssl libssl3 \
    && rm -rf /var/lib/apt/lists/*

ARG SERVICE

COPY --from=builder /app/target/release/$SERVICE /usr/local/bin/service

USER appuser

EXPOSE 8080

VOLUME ["/usr/local/data", "/usr/local/logs"]

CMD ["/usr/local/bin/service"]