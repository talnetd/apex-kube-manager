//! Golden fixture runner for `spec/triage/fixtures/` (shared with iOS and
//! Android). Each `<name>.input.json` is a raw Kubernetes object as the API
//! returns it; `<name>.expected.json` is the exact verdict. Every fixture is
//! evaluated at the `now` in `fixtures/meta.json`, so time windows are pinned.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use k8s_openapi::jiff::Timestamp;
use serde_json::Value;

use super::rules::*;
use super::Verdict;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../spec/triage/fixtures")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn evaluate(input: Value, now: Timestamp, path: &Path) -> Verdict {
    let kind = input
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or_else(|| panic!("{}: input has no kind", path.display()))
        .to_string();
    macro_rules! decode {
        ($t:ty) => {
            serde_json::from_value::<$t>(input)
                .unwrap_or_else(|e| panic!("{}: decode {kind}: {e}", path.display()))
        };
    }
    match kind.as_str() {
        "Pod" => pod_verdict(&decode!(k8s_openapi::api::core::v1::Pod), now),
        "Deployment" => deployment_verdict(&decode!(k8s_openapi::api::apps::v1::Deployment), now),
        "StatefulSet" => statefulset_verdict(&decode!(k8s_openapi::api::apps::v1::StatefulSet), now),
        "DaemonSet" => daemonset_verdict(&decode!(k8s_openapi::api::apps::v1::DaemonSet), now),
        "ReplicaSet" => replicaset_verdict(&decode!(k8s_openapi::api::apps::v1::ReplicaSet), now),
        "Job" => job_verdict(&decode!(k8s_openapi::api::batch::v1::Job), now),
        "Node" => node_verdict(&decode!(k8s_openapi::api::core::v1::Node), now),
        "PersistentVolumeClaim" => {
            pvc_verdict(&decode!(k8s_openapi::api::core::v1::PersistentVolumeClaim), now)
        }
        other => panic!("{}: no rules for kind {other}", path.display()),
    }
}

/// Rule ids the desktop implements; each must be exercised by a fixture.
const IMPLEMENTED_RULES: &[&str] = &[
    "TRI-POD-000", "TRI-POD-001", "TRI-POD-002", "TRI-POD-003", "TRI-POD-004", "TRI-POD-005",
    "TRI-POD-006", "TRI-POD-007", "TRI-POD-008", "TRI-POD-009", "TRI-POD-010", "TRI-POD-011",
    "TRI-POD-012", "TRI-WL-000", "TRI-WL-001", "TRI-WL-002", "TRI-WL-003", "TRI-WL-004",
    "TRI-WL-005", "TRI-JOB-000", "TRI-JOB-001", "TRI-JOB-003", "TRI-NODE-000", "TRI-NODE-001",
    "TRI-NODE-002", "TRI-NODE-003", "TRI-NODE-004", "TRI-PVC-000", "TRI-PVC-001", "TRI-PVC-002",
];

#[test]
fn golden_fixtures_match() {
    let dir = fixtures_dir();
    let meta = read_json(&dir.join("meta.json"));
    let now: Timestamp = meta["now"]
        .as_str()
        .expect("meta.json: now")
        .parse()
        .expect("meta.json: now is RFC 3339");

    let mut files = Vec::new();
    walk(&dir, &mut files);
    files.sort();

    let mut failures = Vec::new();
    let mut seen_rules = BTreeSet::new();
    let mut count = 0;
    for input in files.iter().filter(|p| p.to_string_lossy().ends_with(".input.json")) {
        let expected_path = PathBuf::from(input.to_string_lossy().replace(".input.json", ".expected.json"));
        assert!(expected_path.exists(), "{}: missing .expected.json", input.display());
        let expected = read_json(&expected_path);
        let got = serde_json::to_value(evaluate(read_json(input), now, input)).unwrap();
        if let Some(r) = got.get("ruleId").and_then(|r| r.as_str()) {
            seen_rules.insert(r.to_string());
        }
        if got != expected {
            failures.push(format!(
                "{}\n  expected: {expected}\n  got:      {got}",
                input.strip_prefix(&dir).unwrap_or(input).display()
            ));
        }
        count += 1;
    }

    // An expected file without an input would silently never run.
    for exp in files.iter().filter(|p| p.to_string_lossy().ends_with(".expected.json")) {
        let input = PathBuf::from(exp.to_string_lossy().replace(".expected.json", ".input.json"));
        assert!(input.exists(), "{}: orphan expected file", exp.display());
    }

    assert!(count >= 25, "expected at least 25 fixtures, found {count}");
    assert!(failures.is_empty(), "{} fixture(s) failed:\n{}", failures.len(), failures.join("\n"));

    let missing: Vec<_> = IMPLEMENTED_RULES.iter().filter(|r| !seen_rules.contains(**r)).collect();
    assert!(missing.is_empty(), "rules without a fixture: {missing:?}");
}

#[test]
fn verdict_serializes_to_the_shared_schema_names() {
    let v = Verdict::fail("TRI-POD-003", super::Severity::Error, "CrashLoopBackOff")
        .since(Some("2026-10-28T11:50:00Z".into()));
    assert_eq!(
        serde_json::to_value(&v).unwrap(),
        serde_json::json!({
            "status": "fail", "reason": "CrashLoopBackOff", "ruleId": "TRI-POD-003",
            "since": "2026-10-28T11:50:00Z", "severity": "error"
        })
    );
    let p = Verdict::progressing("TRI-WL-005", "Rolling out");
    assert_eq!(serde_json::to_value(&p).unwrap()["tone"], "progressing");
}
