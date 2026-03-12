# --- Build Stage ---
FROM rust:1.94-slim-bookworm as builder

WORKDIR /app

# Install build dependencies (OpenSSL is needed for reqwest/mongodb)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests to cache dependencies
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to pre-build dependencies (saves time on future builds)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/hope_backend_rust*

# Now copy the actual source and build the real app
COPY config ./config
COPY src ./src
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/hope-backend-rust .

# Render uses the PORT environment variable (defaulting to 8080 or 10000)
EXPOSE 8080

# Run the app
CMD ["./hope-backend-rust"]
