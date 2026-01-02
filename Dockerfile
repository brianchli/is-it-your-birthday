# built with snipp
# https://leapcell.io/blog/building-minimal-and-efficient-rust-web-app-docker-images

# Stage 1
FROM rust:1.92-slim-bookworm as BUILDER

WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates

COPY Cargo.toml Cargo.lock ./

# cached layer if no changes are made to build dependencies.
RUN cargo fetch --locked --target x86_64-unknown-linux-musl

COPY src ./src

# Build the release binary with musl target
# --release for optimizations and smaller size
# --locked to ensure reproducible builds based on Cargo.lock
# --target for static linking with musl libc
RUN CARGO_INCREMENTAL=0 \
    RUSTFLAGS="-C strip=debuginfo -C target-feature=+aes,+sse2,+ssse3" \
    cargo build --release --locked --target x86_64-unknown-linux-musl

# Stage 2
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/my_app /app/birthdays
USER 1001

EXPOSE ${BACKEND_PORT}
ENTRYPOINT ["./app/birthdays"]
