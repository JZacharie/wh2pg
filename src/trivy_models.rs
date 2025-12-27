use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Generic Trivy Report Structure
#[derive(Debug, Serialize, Deserialize)]
pub struct TrivyWebhookPayload {
    #[serde(rename = "type")]
    pub report_type: String,  // VulnerabilityReport, ConfigAuditReport, RbacAssessmentReport, ExposedSecretReport, ClusterComplianceReport
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
    pub report: serde_json::Value,  // Full report as JSON
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}

// Vulnerability Report
#[derive(Debug, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub scanner: Scanner,
    pub registry: Registry,
    pub artifact: Artifact,
    pub summary: Summary,
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scanner {
    pub name: String,
    pub vendor: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub server: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub repository: String,
    pub tag: Option<String>,
    pub digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    #[serde(rename = "criticalCount")]
    pub critical_count: i32,
    #[serde(rename = "highCount")]
    pub high_count: i32,
    #[serde(rename = "mediumCount")]
    pub medium_count: i32,
    #[serde(rename = "lowCount")]
    pub low_count: i32,
    #[serde(rename = "unknownCount")]
    pub unknown_count: i32,
    #[serde(rename = "noneCount")]
    pub none_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    #[serde(rename = "vulnerabilityID")]
    pub vulnerability_id: String,
    pub resource: String,
    #[serde(rename = "installedVersion")]
    pub installed_version: String,
    #[serde(rename = "fixedVersion")]
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: Option<String>,
    #[serde(rename = "primaryLink")]
    pub primary_link: Option<String>,
    pub score: Option<f64>,
}

// Config Audit Report
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigAuditReport {
    pub scanner: Scanner,
    pub summary: ConfigAuditSummary,
    pub checks: Vec<ConfigCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigAuditSummary {
    #[serde(rename = "criticalCount")]
    pub critical_count: i32,
    #[serde(rename = "highCount")]
    pub high_count: i32,
    #[serde(rename = "mediumCount")]
    pub medium_count: i32,
    #[serde(rename = "lowCount")]
    pub low_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigCheck {
    #[serde(rename = "checkID")]
    pub check_id: String,
    pub title: String,
    pub severity: String,
    pub category: String,
    pub messages: Vec<String>,
    pub success: bool,
}

// RBAC Assessment Report
#[derive(Debug, Serialize, Deserialize)]
pub struct RbacAssessmentReport {
    pub scanner: Scanner,
    pub summary: ConfigAuditSummary,
    pub checks: Vec<ConfigCheck>,
}

// Exposed Secret Report
#[derive(Debug, Serialize, Deserialize)]
pub struct ExposedSecretReport {
    pub scanner: Scanner,
    pub summary: SecretSummary,
    pub secrets: Vec<ExposedSecret>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretSummary {
    #[serde(rename = "criticalCount")]
    pub critical_count: i32,
    #[serde(rename = "highCount")]
    pub high_count: i32,
    #[serde(rename = "mediumCount")]
    pub medium_count: i32,
    #[serde(rename = "lowCount")]
    pub low_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExposedSecret {
    pub target: String,
    #[serde(rename = "ruleID")]
    pub rule_id: String,
    pub title: String,
    pub severity: String,
    pub category: String,
    pub match: String,
}

// Cluster Compliance Report
#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterComplianceReport {
    pub name: String,
    pub title: String,
    pub summary: ComplianceSummary,
    #[serde(rename = "controlChecks")]
    pub control_checks: Vec<ControlCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceSummary {
    #[serde(rename = "passCount")]
    pub pass_count: i32,
    #[serde(rename = "failCount")]
    pub fail_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlCheck {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub total_fail: Option<i32>,
}
