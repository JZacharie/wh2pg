# Changelog - Intégration Trivy

## Version 2.0.0 - 2025-12-27

### 🎯 Objectif
Intégration complète de Trivy Operator avec wh2pg pour stocker les rapports de sécurité dans PostgreSQL.

### ✨ Nouvelles Fonctionnalités

#### 1. Support des Rapports Trivy
- **Endpoint dédié** : `/webhook/trivy` pour recevoir les rapports Trivy
- **5 types de rapports supportés** :
  - `VulnerabilityReport` - Vulnérabilités CVE dans les images
  - `ConfigAuditReport` - Audits de configuration Kubernetes
  - `RbacAssessmentReport` - Évaluations RBAC
  - `ExposedSecretReport` - Secrets exposés
  - `ClusterComplianceReport` - Conformité CIS/NSA/PSS

#### 2. Nouveaux Modules Rust
- `src/trivy_models.rs` - Modèles de données pour les rapports Trivy
- `src/trivy_handlers.rs` - Handlers HTTP et logique de stockage

#### 3. Schémas de Base de Données
Création de 5 tables PostgreSQL optimisées :
- `trivy_vulnerability_reports` - Rapports de vulnérabilités
- `trivy_configaudit_reports` - Audits de configuration
- `trivy_rbac_reports` - Évaluations RBAC
- `trivy_secret_reports` - Secrets exposés
- `trivy_compliance_reports` - Rapports de conformité

**Caractéristiques** :
- Indexes GIN sur JSONB pour recherches rapides
- Indexes composites sur sévérité et namespace
- Contraintes UNIQUE sur `report_uid`
- Support UPSERT (ON CONFLICT)

#### 4. Bases de Données pg-prd
Ajout de 5 nouvelles bases dans `/home/joseph/git/jo3/Applications/pg-prd/Databases.yaml` :
- `trivy_vulnerabilities`
- `trivy_configaudit`
- `trivy_rbac`
- `trivy_secrets`
- `trivy_compliance`

### 📝 Documentation

#### README.md
- Section complète sur l'intégration Trivy
- Exemples de requêtes webhook
- Schémas de base de données
- Requêtes SQL utiles pour l'analyse
- Guide de visualisation avec Grafana

#### Exemples SQL
- `examples/trivy-init.sql` - Script d'initialisation des tables
- `examples/trivy-queries.sql` - 20+ requêtes d'analyse prêtes à l'emploi

### ⚙️ Configuration

#### wh2pg (jo3)
- **Activé** : Application ArgoCD activée
- **Ingress** : `wh2pg.p.zacharie.org` avec Traefik
- **ServiceMonitor** : Prometheus activé
- **Base de données** : `cluster-pg-rw.pg-prd.svc/wh2pg`

#### Trivy Operator
- **Webhook URL** : Mise à jour vers `https://wh2pg.p.zacharie.org/webhook/trivy`
- **Timeout** : 30s
- **Deleted Reports** : Activé

### 🔍 Requêtes SQL Utiles

#### Top Images Vulnérables
```sql
SELECT image_repository, image_tag, critical_count, high_count
FROM trivy_vulnerability_reports
ORDER BY critical_count DESC, high_count DESC
LIMIT 10;
```

#### Sécurité par Namespace
```sql
SELECT namespace, 
       SUM(critical_count) as critical,
       SUM(high_count) as high
FROM trivy_vulnerability_reports
GROUP BY namespace
ORDER BY critical DESC;
```

#### Secrets Exposés
```sql
SELECT namespace, name, critical_count + high_count as severe
FROM trivy_secret_reports
WHERE critical_count > 0 OR high_count > 0
ORDER BY severe DESC;
```

### 📊 Vues Créées
- `latest_vulnerability_reports` - Derniers rapports par image
- `namespace_security_summary` - Résumé sécurité par namespace
- `top_vulnerable_images` - Top 50 images vulnérables

### 🚀 Déploiement

1. **Appliquer les bases de données** :
   ```bash
   kubectl apply -f Applications/pg-prd/Databases.yaml
   ```

2. **Déployer wh2pg** :
   ```bash
   # ArgoCD sync automatique
   kubectl get app wh2pg -n argocd
   ```

3. **Mettre à jour Trivy Operator** :
   ```bash
   # ArgoCD sync automatique
   kubectl get app trivy-operator -n argocd
   ```

### 🔧 Maintenance

#### Nettoyage des Anciens Rapports
```sql
-- Supprimer les rapports de plus de 90 jours
DELETE FROM trivy_vulnerability_reports 
WHERE received_at < NOW() - INTERVAL '90 days';
```

#### Statistiques de Stockage
```sql
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE tablename LIKE 'trivy_%'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### 📈 Métriques Prometheus
Le ServiceMonitor expose :
- `wh2pg_requests_total{endpoint="/webhook/trivy"}`
- `wh2pg_requests_duration_seconds`
- `wh2pg_db_connections`

### 🔐 Sécurité
- TLS activé via cert-manager
- Authentification PostgreSQL via secrets
- Rate limiting via Traefik
- Validation des payloads JSON

### 🐛 Problèmes Connus
Aucun

### 📚 Références
- [Trivy Operator Docs](https://aquasecurity.github.io/trivy-operator/)
- [wh2pg README](./README.md)
- [PostgreSQL JSONB](https://www.postgresql.org/docs/current/datatype-json.html)

### 👥 Contributeurs
- Configuration initiale : @JZacharie
- Intégration Trivy : Assistant AI

---

## Prochaines Étapes

### Court Terme
- [ ] Créer dashboard Grafana pour visualisation
- [ ] Configurer alertes sur vulnérabilités critiques
- [ ] Ajouter rétention automatique des données

### Moyen Terme
- [ ] API REST pour requêter les rapports
- [ ] Export des rapports en PDF
- [ ] Intégration avec Slack/Teams pour notifications

### Long Terme
- [ ] Machine Learning pour prédiction de vulnérabilités
- [ ] Recommandations automatiques de remédiation
- [ ] Dashboard temps réel avec WebSockets
