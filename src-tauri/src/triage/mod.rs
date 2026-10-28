//! Shared triage rules (spec/triage/rules.md, redesign spec 6.10 and 11).
//!
//! Desktop computes health here, in Rust only: every list `*Info` struct
//! carries a `verdict` and the frontend maps it to display (health.ts) without
//! re-deriving anything. The same rules are implemented by the iOS and Android
//! apps; the golden fixtures in `spec/triage/fixtures/` keep all three honest.
//!
//! Rules are pure functions over decoded Kubernetes objects plus a `now`, so
//! fixtures can pin time and the functions stay trivially testable.

pub mod rules;

#[cfg(test)]
mod fixtures;

pub use rules::{
    daemonset_verdict, deployment_verdict, job_verdict, node_verdict, pod_verdict, pvc_verdict,
    replicaset_verdict, statefulset_verdict,
};

use serde::{Deserialize, Serialize};

/// Verdict status. Names are part of the shared contract (verdict.schema.json).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Warn,
    Fail,
    Unknown,
}

/// Present when status is `fail` (critical or error) or `warn` (warning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Error,
    Warning,
}

/// Only with status `ok`: lets every platform derive the six display states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Progressing,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Verdict {
    pub status: Status,
    pub reason: String,
    pub rule_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
}

impl Verdict {
    pub fn ok(rule: &str, reason: impl Into<String>) -> Self {
        Self::new(Status::Ok, rule, reason, None, None)
    }
    pub fn progressing(rule: &str, reason: impl Into<String>) -> Self {
        let mut v = Self::new(Status::Ok, rule, reason, None, None);
        v.tone = Some(Tone::Progressing);
        v
    }
    pub fn neutral(rule: &str, reason: impl Into<String>) -> Self {
        let mut v = Self::new(Status::Ok, rule, reason, None, None);
        v.tone = Some(Tone::Neutral);
        v
    }
    pub fn warn(rule: &str, reason: impl Into<String>) -> Self {
        Self::new(Status::Warn, rule, reason, Some(Severity::Warning), None)
    }
    pub fn fail(rule: &str, severity: Severity, reason: impl Into<String>) -> Self {
        Self::new(Status::Fail, rule, reason, Some(severity), None)
    }
    pub fn unknown(rule: &str, reason: impl Into<String>) -> Self {
        Self::new(Status::Unknown, rule, reason, None, None)
    }

    fn new(
        status: Status,
        rule: &str,
        reason: impl Into<String>,
        severity: Option<Severity>,
        since: Option<String>,
    ) -> Self {
        Self {
            status,
            reason: reason.into(),
            rule_id: rule.to_string(),
            since,
            severity,
            tone: None,
        }
    }

    pub fn since(mut self, since: Option<String>) -> Self {
        self.since = since;
        self
    }

    /// Lower is worse. Picks the single verdict when several rules match.
    pub fn badness(&self) -> u8 {
        match (self.status, self.severity, self.tone) {
            (Status::Fail, Some(Severity::Critical), _) => 0,
            (Status::Fail, _, _) => 1,
            (Status::Warn, _, _) => 2,
            (Status::Unknown, _, _) => 3,
            (Status::Ok, _, Some(Tone::Progressing)) => 4,
            (Status::Ok, _, None) => 5,
            (Status::Ok, _, Some(Tone::Neutral)) => 6,
        }
    }

    /// Pulse OK:FAIL counting: only `fail` counts as failing.
    pub fn is_fail(&self) -> bool {
        self.status == Status::Fail
    }
}
