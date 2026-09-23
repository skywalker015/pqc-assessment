# PQC Rules Catalog

This document defines the initial set of rules that the Configurable Rule Engine will evaluate to determine an asset's Post-Quantum Cryptography (PQC) readiness.

## Rule 1: Legacy TLS Versions
- **ID:** `PQC-TLS-001`
- **Description:** Flags any asset negotiating TLS 1.1 or TLS 1.0. 
- **Severity:** CRITICAL
- **Remediation:** Enforce TLS 1.3 or TLS 1.2 at a minimum.

## Rule 2: Non-Hybrid Key Exchange
- **ID:** `PQC-TLS-002`
- **Description:** Flags TLS 1.3 connections that rely purely on classical elliptic curves (e.g., X25519) without a hybrid post-quantum mechanism (e.g., X25519+ML-KEM).
- **Severity:** HIGH
- **Remediation:** Update the TLS termination proxy (e.g., Nginx, Envoy) to support hybrid PQC key encapsulation.

## Rule 3: Outdated OpenSSH Version
- **ID:** `PQC-SSH-001`
- **Description:** Flags OpenSSH versions older than 9.0, which lack support for the `sntrup761x25519-sha512@openssh.com` hybrid key exchange.
- **Severity:** MEDIUM
- **Remediation:** Upgrade OpenSSH server to >= 9.0.

## Rule 4: Weak Asymmetric Keys
- **ID:** `PQC-KEY-001`
- **Description:** Flags the presence of RSA keys < 3072-bit or any SHA-1 certificates in the trust store.
- **Severity:** HIGH
- **Remediation:** Rotate keys to stronger classical parameters (RSA >= 3072, ECC >= 256) while transitioning to ML-DSA signatures.

