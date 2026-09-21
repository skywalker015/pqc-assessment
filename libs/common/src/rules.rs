use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PqcObservationSummary {
    pub protocol: String,
    pub tls_version: Option<String>,
    pub ssh_version: Option<String>,
    pub key_type: Option<String>,
    pub weak_crypto_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcAssessmentResult {
    pub score: i32,
    pub status: String,
    pub findings: Vec<String>,
}

pub fn evaluate_observation(summary: &PqcObservationSummary) -> PqcAssessmentResult {
    let mut findings = Vec::new();
    let mut score = 100;

    if summary.protocol.eq_ignore_ascii_case("tls") {
        if summary
            .tls_version
            .as_deref()
            .map(|v| v.starts_with("1.0") || v.starts_with("1.1"))
            .unwrap_or(false)
        {
            findings.push("Legacy TLS version detected".to_string());
            score -= 35;
        }
    }

    if summary.protocol.eq_ignore_ascii_case("ssh") {
        if summary
            .ssh_version
            .as_deref()
            .map(|v| v.contains("OpenSSH_7") || v.contains("OpenSSH_8.0") || v.contains("OpenSSH_8.1"))
            .unwrap_or(false)
        {
            findings.push("Legacy SSH version discovered".to_string());
            score -= 25;
        }
    }

    if summary.weak_crypto_detected {
        findings.push("Weak or legacy cryptography present".to_string());
        score -= 30;
    }

    if summary.key_type.as_deref() == Some("RSA") {
        findings.push("RSA key material should be reviewed for PQC migration".to_string());
        score -= 10;
    }

    if findings.is_empty() {
        score = 100;
    }

    let status = if score >= 85 {
        "ready".to_string()
    } else if score >= 60 {
        "needs_attention".to_string()
    } else {
        "at_risk".to_string()
    };

    PqcAssessmentResult { score: score.max(0), status, findings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_tls_is_flagged() {
        let result = evaluate_observation(&PqcObservationSummary {
            protocol: "tls".to_string(),
            tls_version: Some("1.0".to_string()),
            ssh_version: None,
            key_type: None,
            weak_crypto_detected: false,
        });

        assert!(result.score < 100);
        assert!(result.status != "ready");
        assert!(result.findings.iter().any(|f| f.contains("Legacy TLS version")));
    }
}
