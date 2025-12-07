# Security Audit Report

**Date:** 2025-12-07
**Project:** Apex Kube Manager
**Auditor:** Automated Security Review

## Executive Summary

Security audit of the Apex Kube Manager codebase identified **8 security findings** across multiple categories, ranging from Critical to Low severity.

---

## Findings

### 1. Content Security Policy (CSP) Disabled

| Attribute | Value |
|-----------|-------|
| **Severity** | CRITICAL |
| **Location** | `src-tauri/tauri.conf.json:28` |
| **CWE** | CWE-693, CWE-79 |

**Description:**
The Content Security Policy is explicitly disabled by setting `"csp": null`. This removes a critical layer of defense against Cross-Site Scripting (XSS) attacks.

**Remediation:**
```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' ipc: http://ipc.localhost"
}
```

---

### 2. Kubeconfig File Write Without Atomic Operations

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **Location** | `src-tauri/src/kubernetes.rs:129-131` |
| **CWE** | CWE-367, CWE-459 |

**Description:**
The `switch_to_context` function directly overwrites the kubeconfig file without using atomic write operations. If the write fails mid-way, the kubeconfig could be corrupted.

**Remediation:**
Write to a temporary file first, then rename:
```rust
let temp_path = config_path.with_extension("tmp");
std::fs::write(&temp_path, &yaml)?;
std::fs::rename(&temp_path, &config_path)?;
```

---

### 3. Missing Input Validation on Kubernetes Resource Names

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **Location** | `src-tauri/src/commands.rs:51-55` |
| **CWE** | CWE-20 |

**Description:**
The `delete_pod` and `get_pod_logs` commands accept namespace and pod name parameters without validation. Malformed inputs could cause unexpected behavior.

**Remediation:**
Add validation using Kubernetes naming rules:
```rust
static K8S_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9]([-a-z0-9]*[a-z0-9])?$").unwrap()
});

pub fn validate_k8s_name(name: &str, field: &str) -> Result<()> {
    if name.is_empty() || name.len() > 253 {
        return Err(AppError::Custom(format!("{} must be 1-253 characters", field)));
    }
    if !K8S_NAME_REGEX.is_match(name) {
        return Err(AppError::Custom(format!("{} contains invalid characters", field)));
    }
    Ok(())
}
```

---

### 4. Verbose Error Messages May Leak Sensitive Information

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **Location** | `src-tauri/src/error.rs:1-35` |
| **CWE** | CWE-209 |

**Description:**
Error messages are serialized directly to strings and sent to the frontend. Internal error details could expose sensitive information about system configuration.

**Remediation:**
Sanitize errors before sending to frontend:
```rust
let user_message = match self {
    AppError::Kube(_) => "Kubernetes operation failed".to_string(),
    AppError::KubeConfig(_) => "Kubeconfig error".to_string(),
    AppError::Io(_) => "File operation failed".to_string(),
    AppError::Custom(msg) => msg.clone(),
};
```

---

### 5. Missing Rate Limiting on Destructive Operations

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **Location** | `src-tauri/src/commands.rs:51-55` |
| **CWE** | CWE-770, CWE-799 |

**Description:**
The `delete_pod` command has no rate limiting at the backend level. A malicious script or compromised frontend could rapidly delete multiple pods.

**Remediation:**
Implement time-based throttling on destructive operations.

---

### 6. Shell Plugin Enabled with Open Permission

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **Location** | `src-tauri/tauri.conf.json:43-45` |
| **CWE** | CWE-250 |

**Description:**
The shell plugin is enabled with `"open": true`, allowing the application to open URLs with system handlers. Could be abused if combined with other vulnerabilities.

**Remediation:**
Disable or restrict to specific URL patterns:
```json
"plugins": {
  "shell": {
    "open": "^https://(kubernetes\\.io|docs\\.tauri\\.app)"
  }
}
```

---

### 7. Vulnerable npm Dependencies

| Attribute | Value |
|-----------|-------|
| **Severity** | MEDIUM |
| **Location** | `package.json` |
| **CWE** | CWE-1035 |
| **CVE** | GHSA-67mh-4wv8-2f99 |

**Description:**
npm audit identified 4 moderate vulnerabilities in development dependencies (esbuild, vite).

**Remediation:**
```bash
npm update vite @sveltejs/vite-plugin-svelte
```

---

### 8. Kubernetes Client Uses Inferred Configuration Without Verification

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW |
| **Location** | `src-tauri/src/kubernetes.rs:136-140` |
| **CWE** | CWE-295 |

**Description:**
The Kubernetes client trusts whatever kubeconfig the user has configured, including potentially insecure settings like `insecure-skip-tls-verify`.

**Remediation:**
Add a warning when insecure configurations are detected:
```rust
if config.accept_invalid_certs {
    tracing::warn!("TLS certificate verification is disabled for this cluster.");
}
```

---

## Summary

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| 1 | CSP Disabled | CRITICAL | Open |
| 2 | Non-atomic kubeconfig write | HIGH | Open |
| 3 | Missing input validation | MEDIUM | Open |
| 4 | Verbose error messages | MEDIUM | Open |
| 5 | No rate limiting on delete | MEDIUM | Open |
| 6 | Shell plugin overly permissive | MEDIUM | Open |
| 7 | Vulnerable npm dependencies | MEDIUM | Open |
| 8 | No TLS verification warning | LOW | Open |

---

## Positive Security Observations

- No hardcoded secrets or credentials in codebase
- No XSS-prone `{@html}` usage in Svelte components
- Proper Rust error handling with `Result` types
- Frontend confirmation dialogs for destructive operations
- Proper `.gitignore` excluding sensitive files
- Strong typing with TypeScript and Rust

---

## Reporting Security Issues

If you discover a security vulnerability, please report it by opening a private issue or contacting the maintainers directly. Do not disclose security issues publicly until they have been addressed.
