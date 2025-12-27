-- ============================================================================
-- TRIVY SECURITY ANALYSIS QUERIES
-- Collection de requêtes SQL utiles pour analyser les rapports Trivy
-- ============================================================================

-- ============================================================================
-- VULNERABILITY ANALYSIS
-- ============================================================================

-- 1. Top 20 images avec le plus de vulnérabilités critiques
SELECT 
    image_repository,
    image_tag,
    critical_count,
    high_count,
    medium_count,
    low_count,
    received_at
FROM trivy_vulnerability_reports
ORDER BY critical_count DESC, high_count DESC
LIMIT 20;

-- 2. Évolution des vulnérabilités dans le temps (30 derniers jours)
SELECT 
    DATE(received_at) as scan_date,
    COUNT(*) as total_scans,
    SUM(critical_count) as critical,
    SUM(high_count) as high,
    SUM(medium_count) as medium,
    SUM(low_count) as low
FROM trivy_vulnerability_reports
WHERE received_at > NOW() - INTERVAL '30 days'
GROUP BY DATE(received_at)
ORDER BY scan_date DESC;

-- 3. Images sans vulnérabilités critiques ou élevées
SELECT 
    image_repository,
    image_tag,
    medium_count,
    low_count,
    received_at
FROM trivy_vulnerability_reports
WHERE critical_count = 0 AND high_count = 0
ORDER BY received_at DESC;

-- 4. Détail des CVE critiques (extraction depuis JSONB)
SELECT 
    image_repository,
    image_tag,
    jsonb_array_elements(full_report->'vulnerabilities') ->> 'vulnerabilityID' as cve_id,
    jsonb_array_elements(full_report->'vulnerabilities') ->> 'severity' as severity,
    jsonb_array_elements(full_report->'vulnerabilities') ->> 'title' as title,
    jsonb_array_elements(full_report->'vulnerabilities') ->> 'fixedVersion' as fixed_version
FROM trivy_vulnerability_reports
WHERE critical_count > 0
ORDER BY received_at DESC
LIMIT 100;

-- 5. Statistiques par registry
SELECT 
    SPLIT_PART(image_repository, '/', 1) as registry,
    COUNT(*) as image_count,
    SUM(critical_count) as total_critical,
    SUM(high_count) as total_high,
    AVG(critical_count + high_count) as avg_severe_vulns
FROM trivy_vulnerability_reports
GROUP BY SPLIT_PART(image_repository, '/', 1)
ORDER BY total_critical DESC;

-- ============================================================================
-- CONFIG AUDIT ANALYSIS
-- ============================================================================

-- 6. Namespaces avec le plus de problèmes de configuration
SELECT 
    namespace,
    COUNT(*) as total_reports,
    SUM(critical_count) as total_critical,
    SUM(high_count) as total_high,
    SUM(medium_count) as total_medium,
    MAX(received_at) as last_scan
FROM trivy_configaudit_reports
WHERE namespace IS NOT NULL
GROUP BY namespace
ORDER BY total_critical DESC, total_high DESC;

-- 7. Top des checks de configuration échouant
SELECT 
    jsonb_array_elements(full_report->'checks') ->> 'checkID' as check_id,
    jsonb_array_elements(full_report->'checks') ->> 'title' as check_title,
    jsonb_array_elements(full_report->'checks') ->> 'severity' as severity,
    COUNT(*) as failure_count
FROM trivy_configaudit_reports
WHERE jsonb_array_elements(full_report->'checks') ->> 'success' = 'false'
GROUP BY check_id, check_title, severity
ORDER BY failure_count DESC
LIMIT 20;

-- 8. Ressources avec des problèmes critiques de configuration
SELECT 
    name,
    namespace,
    critical_count,
    high_count,
    received_at
FROM trivy_configaudit_reports
WHERE critical_count > 0
ORDER BY critical_count DESC, high_count DESC;

-- ============================================================================
-- RBAC ANALYSIS
-- ============================================================================

-- 9. Problèmes RBAC par namespace
SELECT 
    namespace,
    COUNT(*) as rbac_issues,
    SUM(critical_count) as critical_issues,
    SUM(high_count) as high_issues
FROM trivy_rbac_reports
WHERE namespace IS NOT NULL
GROUP BY namespace
ORDER BY critical_issues DESC;

-- 10. Rôles avec des privilèges excessifs
SELECT 
    name,
    namespace,
    critical_count + high_count as severe_issues,
    received_at
FROM trivy_rbac_reports
WHERE critical_count > 0 OR high_count > 0
ORDER BY severe_issues DESC;

-- ============================================================================
-- EXPOSED SECRETS ANALYSIS
-- ============================================================================

-- 11. Secrets exposés par namespace
SELECT 
    namespace,
    name,
    critical_count,
    high_count,
    medium_count,
    received_at
FROM trivy_secret_reports
WHERE critical_count > 0 OR high_count > 0
ORDER BY critical_count DESC, high_count DESC;

-- 12. Types de secrets exposés
SELECT 
    jsonb_array_elements(full_report->'secrets') ->> 'ruleID' as secret_type,
    jsonb_array_elements(full_report->'secrets') ->> 'severity' as severity,
    COUNT(*) as occurrence_count
FROM trivy_secret_reports
GROUP BY secret_type, severity
ORDER BY occurrence_count DESC;

-- ============================================================================
-- COMPLIANCE ANALYSIS
-- ============================================================================

-- 13. État de conformité global
SELECT 
    name,
    compliance_title,
    pass_count,
    fail_count,
    ROUND(100.0 * pass_count / NULLIF(pass_count + fail_count, 0), 2) as compliance_percentage,
    received_at
FROM trivy_compliance_reports
ORDER BY received_at DESC;

-- 14. Contrôles de conformité échouant le plus souvent
SELECT 
    jsonb_array_elements(full_report->'controlChecks') ->> 'id' as control_id,
    jsonb_array_elements(full_report->'controlChecks') ->> 'name' as control_name,
    jsonb_array_elements(full_report->'controlChecks') ->> 'severity' as severity,
    SUM((jsonb_array_elements(full_report->'controlChecks') ->> 'total_fail')::int) as total_failures
FROM trivy_compliance_reports
GROUP BY control_id, control_name, severity
ORDER BY total_failures DESC
LIMIT 20;

-- ============================================================================
-- CROSS-REPORT ANALYSIS
-- ============================================================================

-- 15. Vue d'ensemble de la sécurité par namespace
SELECT 
    COALESCE(v.namespace, c.namespace, r.namespace, s.namespace) as namespace,
    COUNT(DISTINCT v.name) as vuln_scans,
    SUM(v.critical_count) as vuln_critical,
    SUM(v.high_count) as vuln_high,
    COUNT(DISTINCT c.name) as config_scans,
    SUM(c.critical_count) as config_critical,
    SUM(c.high_count) as config_high,
    COUNT(DISTINCT s.name) as secret_scans,
    SUM(s.critical_count) as secrets_critical
FROM trivy_vulnerability_reports v
FULL OUTER JOIN trivy_configaudit_reports c ON v.namespace = c.namespace
FULL OUTER JOIN trivy_rbac_reports r ON v.namespace = r.namespace
FULL OUTER JOIN trivy_secret_reports s ON v.namespace = s.namespace
WHERE COALESCE(v.namespace, c.namespace, r.namespace, s.namespace) IS NOT NULL
GROUP BY COALESCE(v.namespace, c.namespace, r.namespace, s.namespace)
ORDER BY vuln_critical DESC, config_critical DESC;

-- 16. Ressources avec plusieurs types de problèmes
SELECT 
    v.name,
    v.namespace,
    v.critical_count as vuln_critical,
    c.critical_count as config_critical,
    s.critical_count as secret_critical,
    v.critical_count + COALESCE(c.critical_count, 0) + COALESCE(s.critical_count, 0) as total_critical
FROM trivy_vulnerability_reports v
LEFT JOIN trivy_configaudit_reports c ON v.name = c.name AND v.namespace = c.namespace
LEFT JOIN trivy_secret_reports s ON v.name = s.name AND v.namespace = s.namespace
WHERE v.critical_count > 0 
   OR c.critical_count > 0 
   OR s.critical_count > 0
ORDER BY total_critical DESC;

-- ============================================================================
-- TREND ANALYSIS
-- ============================================================================

-- 17. Tendance des vulnérabilités par image (dernières 7 versions)
WITH ranked_scans AS (
    SELECT 
        image_repository,
        image_tag,
        critical_count,
        high_count,
        received_at,
        ROW_NUMBER() OVER (PARTITION BY image_repository ORDER BY received_at DESC) as rn
    FROM trivy_vulnerability_reports
)
SELECT 
    image_repository,
    image_tag,
    critical_count,
    high_count,
    received_at
FROM ranked_scans
WHERE rn <= 7
ORDER BY image_repository, received_at DESC;

-- 18. Amélioration/Dégradation de la sécurité
WITH latest AS (
    SELECT DISTINCT ON (image_repository, image_tag)
        image_repository,
        image_tag,
        critical_count as latest_critical,
        high_count as latest_high,
        received_at as latest_scan
    FROM trivy_vulnerability_reports
    ORDER BY image_repository, image_tag, received_at DESC
),
previous AS (
    SELECT DISTINCT ON (image_repository, image_tag)
        image_repository,
        image_tag,
        critical_count as previous_critical,
        high_count as previous_high,
        received_at as previous_scan
    FROM trivy_vulnerability_reports
    WHERE received_at < (SELECT MAX(received_at) FROM trivy_vulnerability_reports)
    ORDER BY image_repository, image_tag, received_at DESC
)
SELECT 
    l.image_repository,
    l.image_tag,
    l.latest_critical,
    p.previous_critical,
    l.latest_critical - p.previous_critical as critical_delta,
    l.latest_high - p.previous_high as high_delta,
    CASE 
        WHEN l.latest_critical < p.previous_critical THEN 'IMPROVED'
        WHEN l.latest_critical > p.previous_critical THEN 'DEGRADED'
        ELSE 'STABLE'
    END as trend
FROM latest l
JOIN previous p ON l.image_repository = p.image_repository AND l.image_tag = p.image_tag
WHERE l.latest_critical != p.previous_critical OR l.latest_high != p.previous_high
ORDER BY ABS(l.latest_critical - p.previous_critical) DESC;

-- ============================================================================
-- ALERTING QUERIES
-- ============================================================================

-- 19. Nouvelles vulnérabilités critiques (dernières 24h)
SELECT 
    image_repository,
    image_tag,
    critical_count,
    high_count,
    received_at
FROM trivy_vulnerability_reports
WHERE received_at > NOW() - INTERVAL '24 hours'
  AND critical_count > 0
ORDER BY received_at DESC;

-- 20. Secrets exposés récemment découverts
SELECT 
    namespace,
    name,
    critical_count + high_count as severe_secrets,
    received_at
FROM trivy_secret_reports
WHERE received_at > NOW() - INTERVAL '24 hours'
  AND (critical_count > 0 OR high_count > 0)
ORDER BY severe_secrets DESC;
