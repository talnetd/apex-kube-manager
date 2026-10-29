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

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use k8s_openapi::jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// Remembers when Apex first saw an object *unsettled* (not healthy or
/// neutral), per uid. StatefulSets, ReplicaSets and DaemonSets carry no
/// transition timestamps, so without this a scale-up of a year-old object
/// would look degraded "since creation" and fail instantly.
///
/// The clock only means something while Apex keeps looking: if an object
/// wasn't evaluated for longer than `SIGHTING_GAP_S` (about two polls), the
/// next unsettled sighting starts a fresh clock instead of reusing a stale
/// one and jumping straight to warn.
#[derive(Default)]
pub struct UnsettledTracker {
    /// uid -> (first seen unsettled, last seen unsettled)
    seen: HashMap<String, (Timestamp, Timestamp)>,
}

/// Lists poll every 10s and Pulse every 5s, so 60s is a couple of missed polls.
pub const SIGHTING_GAP_S: i64 = 60;

impl UnsettledTracker {
    /// Evaluate `rule(first_seen)`. The first unsettled observation (or the
    /// first after a gap) starts the clock at `now`; a settled verdict clears it.
    pub fn eval(
        &mut self,
        uid: &str,
        now: Timestamp,
        rule: impl Fn(Option<Timestamp>) -> Verdict,
    ) -> Verdict {
        let first = self
            .seen
            .get(uid)
            .filter(|(_, last)| now.as_second() - last.as_second() <= SIGHTING_GAP_S)
            .map(|(first, _)| *first);
        let v = rule(first);
        if v.badness() >= 5 {
            self.seen.remove(uid);
            return v;
        }
        match first {
            Some(f) => {
                self.seen.insert(uid.to_string(), (f, now));
                v
            }
            None => {
                self.seen.insert(uid.to_string(), (now, now));
                // Anything not looked at for a day is stale; bound the map.
                if self.seen.len() > 10_000 {
                    let cutoff = now.as_second() - 24 * 3600;
                    self.seen.retain(|_, (_, last)| last.as_second() > cutoff);
                }
                rule(Some(now))
            }
        }
    }

    /// Context switch: uids belong to the old cluster.
    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

static UNSETTLED: OnceLock<Mutex<UnsettledTracker>> = OnceLock::new();

fn unsettled() -> std::sync::MutexGuard<'static, UnsettledTracker> {
    UNSETTLED
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Process-wide tracker so lists, watches and Pulse agree on one clock.
/// Objects without a uid are evaluated untracked (creation fallback).
pub fn tracked(
    uid: Option<&str>,
    now: Timestamp,
    rule: impl Fn(Option<Timestamp>) -> Verdict,
) -> Verdict {
    let Some(uid) = uid else { return rule(None) };
    unsettled().eval(uid, now, rule)
}

/// Drop every clock (context switch).
pub fn reset_unsettled() {
    unsettled().clear();
}

/// The tracked entry points desktop uses for kinds without transition times.
pub fn statefulset_tracked(s: &k8s_openapi::api::apps::v1::StatefulSet, now: Timestamp) -> Verdict {
    tracked(s.metadata.uid.as_deref(), now, |seen| {
        statefulset_verdict(s, now, seen)
    })
}

pub fn replicaset_tracked(r: &k8s_openapi::api::apps::v1::ReplicaSet, now: Timestamp) -> Verdict {
    tracked(r.metadata.uid.as_deref(), now, |seen| {
        replicaset_verdict(r, now, seen)
    })
}

pub fn daemonset_tracked(d: &k8s_openapi::api::apps::v1::DaemonSet, now: Timestamp) -> Verdict {
    tracked(d.metadata.uid.as_deref(), now, |seen| {
        daemonset_verdict(d, now, seen)
    })
}

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
