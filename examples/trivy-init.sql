-- Trivy Database Initialization Script
-- This script creates the necessary tables for storing Trivy security reports
-- Execute this on each Trivy database (trivy_vulnerabilities, trivy_configaudit, etc.)

-- ============================================================================
-- VULNERABILITY REPORTS DATABASE (trivy_vulnerabilities)
-- ============================================================================

-- Main vulnerability reports table
CREATE TABLE IF NOT EXISTS trivy_vulnerability_reports (
    id SERIAL PRIMARY KEY,
    report_uid VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255),
    scanner_name VARCHAR(100),
    scanner_version VARCHAR(50),
    image_repository VARCHAR(500),
    image_tag VARCHAR(255),
    image_digest VARCHAR(255),
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    unknown_count INTEGER DEFAULT 0,
    full_report JSONB NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_ip VARCHAR(45),
    headers JSONB
);

-- Indexes for vulnerability reports
CREATE INDEX IF NOT EXISTS idx_vuln_namespace ON trivy_vulnerability_reports(namespace);
CREATE INDEX IF NOT EXISTS idx_vuln_received ON trivy_vulnerability_reports(received_at);
CREATE INDEX IF NOT EXISTS idx_vuln_severity ON trivy_vulnerability_reports(critical_count, high_count);
CREATE INDEX IF NOT EXISTS idx_vuln_image ON trivy_vulnerability_reports(image_repository, image_tag);
CREATE INDEX IF NOT EXISTS idx_vuln_report_gin ON trivy_vulnerability_reports USING GIN(full_report);

-- ============================================================================
-- CONFIG AUDIT REPORTS DATABASE (trivy_configaudit)
-- ============================================================================

CREATE TABLE IF NOT EXISTS trivy_configaudit_reports (
    id SERIAL PRIMARY KEY,
    report_uid VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255),
    scanner_name VARCHAR(100),
    scanner_version VARCHAR(50),
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    full_report JSONB NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_ip VARCHAR(45),
    headers JSONB
);

-- Indexes for config audit reports
CREATE INDEX IF NOT EXISTS idx_config_namespace ON trivy_configaudit_reports(namespace);
CREATE INDEX IF NOT EXISTS idx_config_received ON trivy_configaudit_reports(received_at);
CREATE INDEX IF NOT EXISTS idx_config_severity ON trivy_configaudit_reports(critical_count, high_count);
CREATE INDEX IF NOT EXISTS idx_config_report_gin ON trivy_configaudit_reports USING GIN(full_report);

-- ============================================================================
-- RBAC ASSESSMENT REPORTS DATABASE (trivy_rbac)
-- ============================================================================

CREATE TABLE IF NOT EXISTS trivy_rbac_reports (
    id SERIAL PRIMARY KEY,
    report_uid VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255),
    scanner_name VARCHAR(100),
    scanner_version VARCHAR(50),
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    full_report JSONB NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_ip VARCHAR(45),
    headers JSONB
);

-- Indexes for RBAC reports
CREATE INDEX IF NOT EXISTS idx_rbac_namespace ON trivy_rbac_reports(namespace);
CREATE INDEX IF NOT EXISTS idx_rbac_received ON trivy_rbac_reports(received_at);
CREATE INDEX IF NOT EXISTS idx_rbac_severity ON trivy_rbac_reports(critical_count, high_count);
CREATE INDEX IF NOT EXISTS idx_rbac_report_gin ON trivy_rbac_reports USING GIN(full_report);

-- ============================================================================
-- EXPOSED SECRET REPORTS DATABASE (trivy_secrets)
-- ============================================================================

CREATE TABLE IF NOT EXISTS trivy_secret_reports (
    id SERIAL PRIMARY KEY,
    report_uid VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255),
    scanner_name VARCHAR(100),
    scanner_version VARCHAR(50),
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    full_report JSONB NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_ip VARCHAR(45),
    headers JSONB
);

-- Indexes for secret reports
CREATE INDEX IF NOT EXISTS idx_secret_namespace ON trivy_secret_reports(namespace);
CREATE INDEX IF NOT EXISTS idx_secret_received ON trivy_secret_reports(received_at);
CREATE INDEX IF NOT EXISTS idx_secret_severity ON trivy_secret_reports(critical_count, high_count);
CREATE INDEX IF NOT EXISTS idx_secret_report_gin ON trivy_secret_reports USING GIN(full_report);

-- ============================================================================
-- CLUSTER COMPLIANCE REPORTS DATABASE (trivy_compliance)
-- ============================================================================

CREATE TABLE IF NOT EXISTS trivy_compliance_reports (
    id SERIAL PRIMARY KEY,
    report_uid VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    compliance_title VARCHAR(255),
    pass_count INTEGER DEFAULT 0,
    fail_count INTEGER DEFAULT 0,
    full_report JSONB NOT NULL,
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    source_ip VARCHAR(45),
    headers JSONB
);

-- Indexes for compliance reports
CREATE INDEX IF NOT EXISTS idx_compliance_received ON trivy_compliance_reports(received_at);
CREATE INDEX IF NOT EXISTS idx_compliance_name ON trivy_compliance_reports(name);
CREATE INDEX IF NOT EXISTS idx_compliance_report_gin ON trivy_compliance_reports USING GIN(full_report);

-- ============================================================================
-- USEFUL VIEWS
-- ============================================================================

-- View: Latest vulnerability reports per image
CREATE OR REPLACE VIEW latest_vulnerability_reports AS
SELECT DISTINCT ON (image_repository, image_tag)
    id,
    name,
    namespace,
    image_repository,
    image_tag,
    critical_count,
    high_count,
    medium_count,
    low_count,
    received_at
FROM trivy_vulnerability_reports
ORDER BY image_repository, image_tag, received_at DESC;

-- View: Security summary by namespace
CREATE OR REPLACE VIEW namespace_security_summary AS
SELECT 
    namespace,
    COUNT(DISTINCT name) as resource_count,
    SUM(critical_count) as total_critical,
    SUM(high_count) as total_high,
    SUM(medium_count) as total_medium,
    SUM(low_count) as total_low,
    MAX(received_at) as last_scan
FROM trivy_vulnerability_reports
WHERE namespace IS NOT NULL
GROUP BY namespace
ORDER BY total_critical DESC, total_high DESC;

-- View: Top vulnerable images
CREATE OR REPLACE VIEW top_vulnerable_images AS
SELECT 
    image_repository,
    image_tag,
    critical_count,
    high_count,
    medium_count + low_count as other_count,
    received_at
FROM trivy_vulnerability_reports
WHERE critical_count > 0 OR high_count > 0
ORDER BY critical_count DESC, high_count DESC
LIMIT 50;

-- ============================================================================
-- GRANT PERMISSIONS (adjust as needed)
-- ============================================================================

-- Grant permissions to app user
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO app;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app;
