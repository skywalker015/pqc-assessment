use crate::types::DeviceConfigEvidence;

pub fn validate_device_evidence(evidence: &DeviceConfigEvidence) -> Result<(), String> {
    if evidence.device_id.trim().is_empty() {
        return Err("device_id is required".to_string());
    }

    if evidence.openssl_version.is_none() && evidence.openssh_version.is_none() {
        return Err("at least one crypto version field must be present".to_string());
    }

    Ok(())
}
