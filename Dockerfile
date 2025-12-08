# Multi-stage build pour optimiser la taille de l'image finale

# Stage 1: Builder
FROM rustlang/rust:nightly-bullseye as builder

# Installer les dépendances système nécessaires
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Créer un nouveau projet vide pour cacher les dépendances
WORKDIR /app
RUN cargo init --name wh2pg

# Copier les fichiers de dépendances
COPY Cargo.toml ./

# Build des dépendances seulement (pour cache)
RUN cargo build --release && rm src/*.rs

# Copier le code source
COPY src ./src

# Build de l'application
# Touch main.rs pour forcer la recompilation
RUN touch src/main.rs && \
    cargo build --release

# Stage 2: Runtime
FROM debian:bullseye-slim

# Installer les dépendances runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Créer un utilisateur non-root
RUN useradd -m -u 1000 wh2pg

# Créer le répertoire de travail
WORKDIR /app

# Copier le binaire depuis le builder
COPY --from=builder /app/target/release/wh2pg /app/wh2pg

# Changer le propriétaire
RUN chown -R wh2pg:wh2pg /app

# Utiliser l'utilisateur non-root
USER wh2pg

# Exposer le port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/wh2pg", "health"] || exit 1

# Lancer l'application
CMD ["/app/wh2pg"]
