# Stage 1: Chef - Compute recipe
FROM rustlang/rust:nightly-bullseye AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

# Stage 2: Planner - Create recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build dependencies and application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

# Stage 4: Runtime - Minimal image
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/wh2pg /app/wh2pg

# Expose port
EXPOSE 8080

# Run application
CMD ["/app/wh2pg"]
