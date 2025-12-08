# wh2pg - Webhook to PostgreSQL

Service Rust qui reçoit des webhooks en JSON et les enregistre dans une base de données PostgreSQL.

## 📋 Description

`wh2pg` est un service HTTP léger écrit en Rust qui:
- Écoute les requêtes POST webhook
- Valide et parse le JSON reçu
- Enregistre les données dans PostgreSQL
- Fournit des endpoints de santé pour monitoring

## 🏗️ Architecture

```mermaid
graph LR
    A[Webhook Source] -->|POST JSON| B[wh2pg Service]
    B -->|INSERT| C[(PostgreSQL)]
    D[Kubernetes] -->|Health Check| B
    B -->|Logs| E[Monitoring]
```

## 🚀 Démarrage Rapide

### Prérequis

- Rust 1.70+ (pour développement local)
- PostgreSQL 14+
- Docker (pour déploiement conteneurisé)
- Kubernetes + Helm (pour déploiement K8s)

### Installation Locale

1. **Cloner le repository**
   ```bash
   git clone <repository-url>
   cd wh2pg
   ```

2. **Configurer l'environnement**
   ```bash
   cp .env.example .env
   # Éditer .env avec vos paramètres
   ```

3. **Créer la base de données**
   ```sql
   CREATE DATABASE webhooks;
   CREATE TABLE webhook_events (
       id SERIAL PRIMARY KEY,
       payload JSONB NOT NULL,
       received_at TIMESTAMP DEFAULT NOW(),
       source_ip VARCHAR(45),
       headers JSONB
   );
   CREATE INDEX idx_received_at ON webhook_events(received_at);
   CREATE INDEX idx_payload ON webhook_events USING GIN(payload);
   ```

4. **Compiler et lancer**
   ```bash
   cargo build --release
   cargo run --release
   ```

## ⚙️ Configuration

Toutes les configurations se font via variables d'environnement (fichier `.env`).

### Variables PostgreSQL

| Variable | Description | Exemple | Requis |
|----------|-------------|---------|--------|
| `DATABASE_URL` | URL complète de connexion PostgreSQL | `postgresql://user:pass@localhost:5432/webhooks` | ✅ |
| `DB_HOST` | Hôte PostgreSQL | `localhost` | ✅ |
| `DB_PORT` | Port PostgreSQL | `5432` | ✅ |
| `DB_NAME` | Nom de la base de données | `webhooks` | ✅ |
| `DB_USER` | Utilisateur PostgreSQL | `webhook_user` | ✅ |
| `DB_PASSWORD` | Mot de passe PostgreSQL | `secure_password` | ✅ |
| `DB_POOL_SIZE` | Taille du pool de connexions | `10` | ❌ |
| `DB_SSL_MODE` | Mode SSL (`disable`, `require`) | `require` | ❌ |

### Variables Serveur

| Variable | Description | Exemple | Requis |
|----------|-------------|---------|--------|
| `SERVER_HOST` | Adresse d'écoute | `0.0.0.0` | ❌ |
| `SERVER_PORT` | Port d'écoute | `8080` | ❌ |
| `WORKERS` | Nombre de workers | `4` | ❌ |
| `LOG_LEVEL` | Niveau de log (`debug`, `info`, `warn`, `error`) | `info` | ❌ |

### Variables Optionnelles

| Variable | Description | Exemple | Requis |
|----------|-------------|---------|--------|
| `WEBHOOK_SECRET` | Secret pour validation HMAC | `your-secret-key` | ❌ |
| `MAX_PAYLOAD_SIZE` | Taille max du payload (bytes) | `1048576` | ❌ |
| `ENABLE_METRICS` | Activer les métriques Prometheus | `true` | ❌ |

## 📡 API Endpoints

### POST /webhook

Reçoit un webhook JSON.

**Request:**
```bash
curl -X POST http://localhost:8080/webhook \
  -H "Content-Type: application/json" \
  -d '{
    "event": "user.created",
    "data": {
      "user_id": "12345",
      "email": "user@example.com"
    },
    "timestamp": "2025-12-08T06:00:00Z"
  }'
```

**Response:**
```json
{
  "status": "success",
  "id": 42,
  "message": "Webhook received and stored"
}
```

### GET /health

Endpoint de santé pour Kubernetes.

**Response:**
```json
{
  "status": "healthy",
  "database": "connected",
  "uptime_seconds": 3600
}
```

### GET /metrics

Métriques Prometheus (si `ENABLE_METRICS=true`).

## 🐳 Docker

### Build Local

```bash
docker build -t wh2pg:latest .
```

### Run avec Docker

```bash
docker run -d \
  --name wh2pg \
  -p 8080:8080 \
  --env-file .env \
  wh2pg:latest
```

### Docker Compose

```yaml
version: '3.8'
services:
  wh2pg:
    build: .
    ports:
      - "8080:8080"
    env_file:
      - .env
    depends_on:
      - postgres
  
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: webhooks
      POSTGRES_USER: webhook_user
      POSTGRES_PASSWORD: secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## ☸️ Déploiement Kubernetes

### Avec Helm

1. **Configurer les valeurs**
   ```bash
   cp helm/values.yaml helm/values-prod.yaml
   # Éditer helm/values-prod.yaml
   ```

2. **Installer le chart**
   ```bash
   helm install wh2pg ./helm -f helm/values-prod.yaml
   ```

3. **Mettre à jour**
   ```bash
   helm upgrade wh2pg ./helm -f helm/values-prod.yaml
   ```

4. **Désinstaller**
   ```bash
   helm uninstall wh2pg
   ```

### Configuration Kubernetes

Le chart Helm crée automatiquement:
- **Deployment**: Pods avec l'application
- **Service**: Exposition interne
- **Secret**: Credentials PostgreSQL
- **ConfigMap**: Configuration non-sensible
- **Ingress** (optionnel): Exposition externe

## 🔧 Développement

### Structure du Projet

```
wh2pg/
├── src/
│   ├── main.rs           # Point d'entrée
│   ├── config.rs         # Configuration
│   ├── db.rs             # Connexion PostgreSQL
│   ├── handlers.rs       # Handlers HTTP
│   └── models.rs         # Modèles de données
├── Cargo.toml            # Dépendances Rust
├── Dockerfile            # Image Docker
├── .env.example          # Exemple de configuration
├── .github/
│   └── workflows/
│       └── docker-build.yml  # CI/CD
└── helm/                 # Charts Kubernetes
    ├── Chart.yaml
    ├── values.yaml
    └── templates/
```

### Dépendances Principales

- **actix-web**: Framework HTTP
- **tokio-postgres**: Client PostgreSQL async
- **serde**: Sérialisation JSON
- **dotenv**: Gestion des variables d'environnement
- **tracing**: Logging structuré

### Tests

```bash
# Tests unitaires
cargo test

# Tests d'intégration
cargo test --test integration

# Coverage
cargo tarpaulin --out Html
```

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## 🔄 CI/CD

Le projet utilise GitHub Actions pour:
- Build automatique du Docker image
- Push vers GitHub Container Registry (GHCR)
- Tagging automatique (`latest`, version, commit SHA)
- Cache des layers Docker pour builds rapides

### Utiliser l'image

```bash
docker pull ghcr.io/<username>/wh2pg:latest
```

## 📊 Monitoring

### Logs

Les logs sont au format JSON structuré:
```json
{
  "timestamp": "2025-12-08T06:00:00Z",
  "level": "INFO",
  "message": "Webhook received",
  "payload_size": 256,
  "source_ip": "192.168.1.100"
}
```

### Métriques Prometheus

Métriques disponibles (si activées):
- `wh2pg_requests_total`: Total des requêtes
- `wh2pg_requests_duration_seconds`: Durée des requêtes
- `wh2pg_db_connections`: Connexions DB actives
- `wh2pg_errors_total`: Total des erreurs

## 🔒 Sécurité

### Bonnes Pratiques

1. **Secrets**: Utiliser Kubernetes Secrets pour les credentials
2. **HMAC**: Activer `WEBHOOK_SECRET` pour valider les webhooks
3. **SSL/TLS**: Utiliser `DB_SSL_MODE=require` en production
4. **Rate Limiting**: Configurer un ingress avec rate limiting
5. **Network Policies**: Limiter l'accès réseau aux pods

### Validation HMAC

Si `WEBHOOK_SECRET` est défini, les webhooks doivent inclure un header `X-Webhook-Signature`:

```bash
curl -X POST http://localhost:8080/webhook \
  -H "Content-Type: application/json" \
  -H "X-Webhook-Signature: sha256=<hmac>" \
  -d '{"event": "test"}'
```

## 🐛 Dépannage

### Erreur de connexion PostgreSQL

```bash
# Vérifier la connectivité
psql -h $DB_HOST -U $DB_USER -d $DB_NAME

# Vérifier les logs
kubectl logs -l app=wh2pg
```

### Performance

```bash
# Augmenter le pool de connexions
DB_POOL_SIZE=20

# Augmenter les workers
WORKERS=8
```

## 📝 License

MIT

## 🤝 Contribution

Les contributions sont bienvenues! Merci de:
1. Fork le projet
2. Créer une branche feature (`git checkout -b feature/amazing`)
3. Commit vos changements (`git commit -m 'Add amazing feature'`)
4. Push vers la branche (`git push origin feature/amazing`)
5. Ouvrir une Pull Request
