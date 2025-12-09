# Configuration ArgoCD pour wh2pg

Ce répertoire contient des exemples de configuration ArgoCD pour déployer `wh2pg`.

## Option 1 : OCI Registry (Recommandé)

Utilise le chart Helm depuis GitHub Container Registry (GHCR).

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: wh2pg
  namespace: argocd
spec:
  destination:
    namespace: wh2pg
    server: https://kubernetes.default.svc
  project: default
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
    automated:
      prune: true
      selfHeal: true
  sources:
    # Chart Helm depuis OCI registry
    - chart: wh2pg
      repoURL: oci://ghcr.io/jzacharie/charts
      targetRevision: '*'  # Utilise la dernière version, ou spécifiez '1.0.0'
      helm:
        valueFiles:
          - $values/values/wh2pg/values.yaml
    # Repository Git pour les values personnalisés
    - ref: values
      repoURL: git@github.com:JZacharie/jo3.git
      targetRevision: main
```

**Avantages :**
- ✅ Pas besoin de configurer un repository Helm dans ArgoCD
- ✅ Authentification via le registry (si privé)
- ✅ Plus moderne et standard

## Option 2 : HTTPS Repository (GitHub Pages)

Utilise le chart Helm depuis GitHub Pages.

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: wh2pg
  namespace: argocd
spec:
  destination:
    namespace: wh2pg
    server: https://kubernetes.default.svc
  project: default
  syncPolicy:
    syncOptions:
      - CreateNamespace=true
    automated:
      prune: true
      selfHeal: true
  sources:
    # Chart Helm depuis HTTPS repository
    - chart: wh2pg
      repoURL: https://jzacharie.github.io/wh2pg
      targetRevision: '*'  # Utilise la dernière version, ou spécifiez '1.0.0'
      helm:
        valueFiles:
          - $values/values/wh2pg/values.yaml
    # Repository Git pour les values personnalisés
    - ref: values
      repoURL: git@github.com:JZacharie/jo3.git
      targetRevision: main
```

**Avantages :**
- ✅ Compatible avec tous les outils Helm
- ✅ Facile à déboguer (index.yaml accessible via navigateur)
- ✅ Pas besoin d'authentification pour les repos publics

## Configuration Simple (Sans Values Externes)

Si vous n'avez pas besoin de values personnalisés depuis un autre repository Git :

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: wh2pg
  namespace: argocd
spec:
  destination:
    namespace: wh2pg
    server: https://kubernetes.default.svc
  project: default
  source:
    chart: wh2pg
    repoURL: oci://ghcr.io/jzacharie/charts
    targetRevision: '1.0.0'
    helm:
      values: |
        replicaCount: 2
        image:
          tag: "1.0.0"
        postgresql:
          external:
            host: "postgresql.default.svc.cluster.local"
            database: "webhooks"
```

## Appliquer la Configuration

```bash
# Appliquer la configuration
kubectl apply -f argocd-wh2pg-oci.yaml

# Vérifier le statut
argocd app get wh2pg

# Synchroniser manuellement (si pas d'auto-sync)
argocd app sync wh2pg
```
