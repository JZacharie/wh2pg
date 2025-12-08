# Stage 1: Builder
FROM rustlang/rust:nightly-bullseye as builder
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create empty project for caching dependencies
RUN cargo init --name wh2pg
COPY Cargo.toml ./

# Build dependencies
RUN cargo build --release && rm src/*.rs

# Copy source code
COPY src ./src

# Build application
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime - Minimal image
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/wh2pg /app/wh2pg

# Expose port
EXPOSE 8080

# Run application
CMD ["/app/wh2pg"]
