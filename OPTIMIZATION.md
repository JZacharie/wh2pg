# Optimisations de compilation GitHub Actions pour wh2pg

## Changements effectués

### 1. Workflow GitHub Actions (`docker-build.yml`)
- ✅ **Suppression du build multi-plateforme** : Passage de `linux/amd64,linux/arm64` à `linux/amd64` uniquement
  - Gain estimé : **50-70% de réduction du temps de build**
  - Le build ARM64 doublait quasiment le temps de compilation
- ✅ **Ajout de l'ID au step build** pour référence dans l'attestation
- ✅ **Ajout de build-args** pour `CARGO_INCREMENTAL=1`

### 2. Dockerfile
- ✅ **Changement de l'image de base** : `rustlang/rust:nightly-bullseye` → `rust:1.83-slim-bullseye`
  - Gain estimé : **30-40% de réduction du temps de téléchargement**
  - Version stable au lieu de nightly (plus fiable)
  - Image slim (moins de packages inutiles)
- ✅ **Installation explicite des dépendances** : pkg-config et libssl-dev
- ✅ **Activation de la compilation incrémentale** via `CARGO_INCREMENTAL`
- ✅ **Ajout de `--locked`** au cargo build pour garantir la reproductibilité
- ✅ **Variable d'environnement** `CARGO_NET_GIT_FETCH_WITH_CLI=true` pour améliorer la stabilité

### 3. .dockerignore (nouveau)
- ✅ **Exclusion des fichiers inutiles** du contexte Docker
  - Répertoire target/
  - Documentation
  - Helm charts
  - Exemples
  - Fichiers IDE
  - Gain estimé : **Réduction de 80-90% de la taille du contexte**

### 4. Cargo.toml
- ✅ **Optimisation du profil release** :
  - `lto = "thin"` au lieu de `lto = true` (thin LTO ~2-3x plus rapide que full LTO)
  - `codegen-units = 16` au lieu de `1` (parallélisation de la compilation)
  - Gain estimé : **40-60% de réduction du temps de compilation release**
- ✅ **Ajout d'un profil dev optimisé** pour les builds locales

## Gains attendus

### Avant les optimisations
- Build multi-plateforme : ~15-25 minutes
- Sans cache : ~20-30 minutes

### Après les optimisations
- Premier build (sans cache) : ~8-12 minutes
- Builds suivants (avec cache) : ~3-6 minutes si seulement le code change

### Réduction totale estimée
- **60-75% de réduction du temps de build**
- **Cache GitHub Actions** optimisé pour réutiliser les dépendances

## Stratégie de build multi-plateforme

### Développement (branches main/develop)
- **Plateformes** : `linux/amd64` uniquement
- **Objectif** : Builds rapides pour itération rapide
- **Workflow** : `.github/workflows/docker-build.yml`

### Releases (tags v*)
- **Plateformes** : `linux/amd64` + `linux/arm64`
- **Objectif** : Compatibilité maximale pour production
- **Workflow** : `.github/workflows/release.yml`
- **Compromise** : Temps de build plus long acceptable pour releases officielles

Cette approche offre le meilleur des deux mondes :
- 🚀 Développement rapide avec builds single-platform
- 🌍 Releases complètes avec support ARM64 pour Raspberry Pi, Apple Silicon, etc.


## Test en local

Pour tester la nouvelle configuration Docker :
```bash
cd /home/joseph/git/wh2pg
docker build -t wh2pg:test .
```

Pour mesurer le temps :
```bash
time docker build -t wh2pg:test .
```
