//! Per-object triage rules. Ids, conditions and labels are specified in
//! `spec/triage/rules.md`; keep the two in step (a rule change without a
//! fixture change fails review).

use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{ContainerStatus, Node, PersistentVolumeClaim, Pod};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use k8s_openapi::jiff::Timestamp;

use super::{Severity, Verdict};

// Time windows (rules.md, "Thresholds").
const POD_PENDING_GRACE_S: i64 = 5 * 60;
const POD_NOT_READY_GRACE_S: i64 = 5 * 60;
const OOM_WINDOW_S: i64 = 60 * 60;
const RESTART_WINDOW_S: i64 = 60 * 60;
const RESTART_MIN: i32 = 3;
const WL_UNAVAILABLE_GRACE_S: i64 = 60;
const WL_DEGRADED_GRACE_S: i64 = 3 * 60;
const PVC_PENDING_GRACE_S: i64 = 2 * 60;
const POD_TERMINATING_GRACE_S: i64 = 5 * 60;
const WL_ROLLOUT_DEADLINE_S: i64 = 10 * 60;
const WL_SCALING_DEADLINE_S: i64 = 10 * 60;

/// Failed pods killed from outside; their controller replaces them.
const DISRUPTION_REASONS: &[&str] = &["Evicted", "Terminated", "NodeShutdown", "Preempting"];

const IMAGE_REASONS: &[&str] = &["ImagePullBackOff", "ErrImagePull", "InvalidImageName"];
const CONFIG_REASONS: &[&str] = &[
    "CreateContainerConfigError",
    "CreateContainerError",
    "RunContainerError",
];
const STARTING_REASONS: &[&str] = &["ContainerCreating", "PodInitializing"];

fn fmt(t: &Time) -> String {
    t.0.to_string()
}

/// Seconds from `t` to `now` (negative if `t` is in the future).
fn age_s(t: &Time, now: Timestamp) -> i64 {
    now.as_second() - t.0.as_second()
}

/// Seconds since `t`; a missing timestamp counts as "long ago" (observed now,
/// with no way to tell when it started), so windows don't hide problems.
fn age_or_max(t: Option<&Time>, now: Timestamp) -> i64 {
    t.map(|t| age_s(t, now)).unwrap_or(i64::MAX)
}

/// Pick the worst verdict; ties keep the earlier (rule-table order) one.
fn worst(candidates: Vec<Verdict>) -> Option<Verdict> {
    let mut best: Option<Verdict> = None;
    for c in candidates {
        match &best {
            Some(b) if b.badness() <= c.badness() => {}
            _ => best = Some(c),
        }
    }
    best
}

// ---------------------------------------------------------------- Pods

fn waiting_reason(cs: &ContainerStatus) -> Option<&str> {
    cs.state.as_ref()?.waiting.as_ref()?.reason.as_deref()
}

pub fn pod_verdict(pod: &Pod, now: Timestamp) -> Verdict {
    let status = pod.status.as_ref();
    let phase = status
        .and_then(|s| s.phase.as_deref())
        .filter(|p| !p.is_empty())
        .unwrap_or("Pending");
    let conditions = status.and_then(|s| s.conditions.as_ref());
    let condition = |t: &str| conditions.and_then(|c| c.iter().find(|c| c.type_ == t));
    let ready_since = condition("Ready")
        .and_then(|c| c.last_transition_time.as_ref())
        .map(fmt);
    let start_since = status.and_then(|s| s.start_time.as_ref()).map(fmt);
    let created = pod.metadata.creation_timestamp.as_ref();

    // Phase-terminal outcomes first: these pods aren't coming back.
    match phase {
        "Failed" => {
            let reason = status
                .and_then(|s| s.reason.clone())
                .filter(|r| !r.is_empty());
            // Killed from outside (eviction, node shutdown, preemption): the
            // workload's controller replaces it, so it isn't broken itself.
            if let Some(r) = reason.as_deref().filter(|r| DISRUPTION_REASONS.contains(r)) {
                return Verdict::warn("TRI-POD-010", r).since(start_since);
            }
            if let Some(c) = condition("DisruptionTarget").filter(|c| c.status == "True") {
                let label = c.reason.clone().filter(|r| !r.is_empty());
                return Verdict::warn("TRI-POD-010", label.unwrap_or_else(|| "Disrupted".into()))
                    .since(c.last_transition_time.as_ref().map(fmt).or(start_since));
            }
            let job_owned = pod
                .metadata
                .owner_references
                .as_ref()
                .is_some_and(|o| o.iter().any(|r| r.kind == "Job"));
            if job_owned {
                // The Job's own verdict (TRI-JOB-001) carries the failure.
                return Verdict::neutral("TRI-POD-012", "Failed").since(start_since);
            }
            let since = start_since.or(ready_since);
            return Verdict::fail(
                "TRI-POD-001",
                Severity::Error,
                reason.unwrap_or_else(|| "Failed".into()),
            )
            .since(since);
        }
        "Succeeded" => return Verdict::neutral("TRI-POD-000", "Completed"),
        "Unknown" => {
            return Verdict::fail("TRI-POD-002", Severity::Error, "Unknown").since(ready_since)
        }
        _ => {}
    }

    let init: &[ContainerStatus] = status
        .and_then(|s| s.init_container_statuses.as_deref())
        .unwrap_or(&[]);
    let main: &[ContainerStatus] = status
        .and_then(|s| s.container_statuses.as_deref())
        .unwrap_or(&[]);
    // Init containers get kubectl's `Init:` prefix on the label.
    let all = init
        .iter()
        .map(|c| (c, "Init:"))
        .chain(main.iter().map(|c| (c, "")));

    let mut found: Vec<Verdict> = Vec::new();
    let mut crashloop = false;
    let mut crash_since: Option<Timestamp> = None;
    let mut oom_since: Option<Timestamp> = None;
    let mut restart_since: Option<Timestamp> = None;

    for (cs, prefix) in all {
        let waiting = waiting_reason(cs);
        let last_term = cs.last_state.as_ref().and_then(|s| s.terminated.as_ref());
        let cur_term = cs.state.as_ref().and_then(|s| s.terminated.as_ref());

        if let Some(r) = waiting {
            if r == "CrashLoopBackOff" {
                crashloop = true;
                if let Some(t) = last_term.and_then(|t| t.finished_at.as_ref()) {
                    crash_since = Some(crash_since.map_or(t.0, |c| c.min(t.0)));
                }
                found.push(Verdict::fail(
                    "TRI-POD-003",
                    Severity::Error,
                    format!("{prefix}{r}"),
                ));
            } else if IMAGE_REASONS.contains(&r) {
                found.push(
                    Verdict::fail("TRI-POD-004", Severity::Error, format!("{prefix}{r}"))
                        .since(ready_since.clone()),
                );
            } else if CONFIG_REASONS.contains(&r) {
                found.push(
                    Verdict::fail("TRI-POD-005", Severity::Error, format!("{prefix}{r}"))
                        .since(ready_since.clone()),
                );
            } else if STARTING_REASONS.contains(&r) {
                found.push(Verdict::progressing("TRI-POD-011", format!("{prefix}{r}")));
            }
        }

        // OOMKilled as the current or last termination, within the window.
        for term in [cur_term, last_term].into_iter().flatten() {
            if term.reason.as_deref() == Some("OOMKilled") {
                if let Some(t) = term.finished_at.as_ref() {
                    if age_s(t, now) <= OOM_WINDOW_S {
                        oom_since = Some(oom_since.map_or(t.0, |o| o.min(t.0)));
                    }
                }
            }
        }

        if let Some(t) = last_term.and_then(|t| t.finished_at.as_ref()) {
            if age_s(t, now) <= RESTART_WINDOW_S {
                restart_since = Some(restart_since.map_or(t.0, |r| r.min(t.0)));
            }
        }
    }

    if let Some(t) = oom_since {
        let since = Some(t.to_string());
        if crashloop {
            // OOM is the cause of the crash loop: report the cause.
            found.insert(
                0,
                Verdict::fail("TRI-POD-006", Severity::Error, "OOMKilled").since(since),
            );
        } else {
            found.push(Verdict::warn("TRI-POD-006", "OOMKilled").since(since));
        }
    }
    // CrashLoopBackOff's since: when the pod stopped being Ready (the loop's
    // start), not the latest crash; fall back to the earliest last finish.
    let not_ready_since = condition("Ready")
        .filter(|c| c.status != "True")
        .and_then(|c| c.last_transition_time.as_ref())
        .map(fmt);
    for v in found.iter_mut().filter(|v| v.rule_id == "TRI-POD-003") {
        v.since = not_ready_since
            .clone()
            .or_else(|| crash_since.map(|t| t.to_string()));
    }

    let restarts: i32 = main
        .iter()
        .chain(init.iter())
        .map(|c| c.restart_count)
        .sum();
    if restarts >= RESTART_MIN {
        if let Some(t) = restart_since {
            found.push(
                Verdict::warn("TRI-POD-009", format!("{restarts} restarts"))
                    .since(Some(t.to_string())),
            );
        }
    }

    // deletionTimestamp is the deadline (request time + grace period), so this
    // measures time past the point the pod should have been gone.
    if let Some(t) = pod.metadata.deletion_timestamp.as_ref() {
        if age_s(t, now) > POD_TERMINATING_GRACE_S {
            found.push(Verdict::warn("TRI-POD-013", "Stuck terminating").since(Some(fmt(t))));
        } else {
            found.push(Verdict::progressing("TRI-POD-011", "Terminating"));
        }
    }

    match phase {
        "Running" => {
            let total = main.len();
            let ready = main.iter().filter(|c| c.ready).count();
            if ready < total {
                let label = format!("{ready}/{total} ready");
                let ready_cond = condition("Ready");
                if age_or_max(
                    ready_cond.and_then(|c| c.last_transition_time.as_ref()),
                    now,
                ) > POD_NOT_READY_GRACE_S
                {
                    found.push(Verdict::warn("TRI-POD-007", label).since(ready_since.clone()));
                } else {
                    found.push(Verdict::progressing("TRI-POD-011", label));
                }
            } else {
                found.push(Verdict::ok("TRI-POD-000", "Running"));
            }
        }
        _ => {
            // Pending (or an unrecognised phase, treated as Pending).
            let created_since = created.map(fmt);
            let unschedulable = condition("PodScheduled").is_some_and(|c| {
                c.status == "False" && c.reason.as_deref() == Some("Unschedulable")
            });
            if age_or_max(created, now) > POD_PENDING_GRACE_S {
                if unschedulable {
                    found.push(
                        Verdict::fail("TRI-POD-008", Severity::Error, "Unschedulable")
                            .since(created_since),
                    );
                } else {
                    found.push(Verdict::warn("TRI-POD-008", "Pending").since(created_since));
                }
            } else {
                found.push(Verdict::progressing("TRI-POD-011", "Pending"));
            }
        }
    }

    worst(found).unwrap_or_else(|| Verdict::unknown("TRI-POD-000", "Unknown"))
}

// ---------------------------------------------------------------- Workloads

/// Shared replica logic for Deployment, StatefulSet and ReplicaSet (TRI-WL-000,
/// -001, -002, -005, -006). Ages are seconds since the clock the caller picked
/// (a condition transition, or first-seen-unsettled; see `unsettled_age`).
struct Replicas<'a> {
    desired: i32,
    available: i32,
    rolling: bool,
    /// `status.replicas < desired`: pods not created yet (scale-up).
    scaling: bool,
    paused: bool,
    /// Healthy state word: "Available" or "Ready".
    ok_word: &'a str,
    /// "available" or "ready" in `n/m ...` labels.
    count_word: &'a str,
    unavailable_age: i64,
    degraded_age: i64,
    /// None = the kind has its own deadline (Deployment: TRI-WL-003).
    rollout_age: Option<i64>,
    unavailable_since: Option<String>,
    degraded_since: Option<String>,
}

fn replica_verdict(r: Replicas) -> Verdict {
    if r.desired == 0 {
        return Verdict::neutral("TRI-WL-000", "Scaled to 0");
    }
    let label = format!("{}/{} {}", r.available, r.desired, r.count_word);
    if r.available == 0 {
        if r.unavailable_age > WL_UNAVAILABLE_GRACE_S {
            return Verdict::fail("TRI-WL-001", Severity::Critical, "Unavailable")
                .since(r.unavailable_since);
        }
        let word = if r.rolling {
            "Rolling out"
        } else if r.scaling {
            "Scaling"
        } else {
            "Starting"
        };
        return Verdict::progressing("TRI-WL-005", word);
    }
    if r.paused {
        return Verdict::neutral("TRI-WL-000", "Paused");
    }
    if r.rolling {
        if r.rollout_age.is_some_and(|a| a > WL_ROLLOUT_DEADLINE_S) {
            return Verdict::warn("TRI-WL-006", "Rollout stuck").since(r.degraded_since);
        }
        return Verdict::progressing("TRI-WL-005", "Rolling out");
    }
    if r.scaling {
        if r.degraded_age > WL_SCALING_DEADLINE_S {
            return Verdict::warn("TRI-WL-002", label).since(r.degraded_since);
        }
        return Verdict::progressing("TRI-WL-005", "Scaling");
    }
    if r.available < r.desired {
        if r.degraded_age <= WL_DEGRADED_GRACE_S {
            return Verdict::progressing("TRI-WL-005", label);
        }
        if r.available * 2 <= r.desired {
            return Verdict::fail("TRI-WL-002", Severity::Error, label).since(r.degraded_since);
        }
        return Verdict::warn("TRI-WL-002", label).since(r.degraded_since);
    }
    Verdict::ok("TRI-WL-000", r.ok_word)
}

/// For kinds with no transition timestamps: seconds since Apex first saw the
/// object unsettled (tracked per uid by the caller), else since creation.
fn unsettled_age(
    first_seen: Option<Timestamp>,
    created: Option<&Time>,
    now: Timestamp,
) -> (i64, Option<String>) {
    match first_seen {
        Some(t) => (now.as_second() - t.as_second(), Some(t.to_string())),
        None => (age_or_max(created, now), None),
    }
}

pub fn deployment_verdict(d: &Deployment, now: Timestamp) -> Verdict {
    let spec = d.spec.as_ref();
    let desired = spec.and_then(|s| s.replicas).unwrap_or(1);
    let st = d.status.as_ref();
    let conds = st.and_then(|s| s.conditions.as_ref());
    let cond = |t: &str| conds.and_then(|c| c.iter().find(|c| c.type_ == t));

    if desired > 0 {
        if let Some(p) = cond("Progressing") {
            if p.status == "False" && p.reason.as_deref() == Some("ProgressDeadlineExceeded") {
                return Verdict::fail("TRI-WL-003", Severity::Error, "Rollout stuck")
                    .since(p.last_transition_time.as_ref().map(fmt));
            }
        }
    }

    let available = st.and_then(|s| s.available_replicas).unwrap_or(0);
    let updated = st.and_then(|s| s.updated_replicas).unwrap_or(0);
    let generation = d.metadata.generation.unwrap_or(0);
    let observed = st.and_then(|s| s.observed_generation).unwrap_or(0);
    let rolling = observed < generation || updated < desired;

    // Available=False transition, else creation, measures the zero window.
    let avail = cond("Available");
    let unavail_from = avail
        .filter(|c| c.status != "True")
        .and_then(|c| c.last_transition_time.as_ref())
        .or(d.metadata.creation_timestamp.as_ref());
    // Progressing lastUpdateTime moves whenever new pods become available.
    let degraded_from = cond("Progressing")
        .and_then(|c| c.last_update_time.as_ref())
        .or(d.metadata.creation_timestamp.as_ref());

    replica_verdict(Replicas {
        desired,
        available,
        rolling,
        scaling: false,
        paused: spec.and_then(|s| s.paused).unwrap_or(false),
        ok_word: "Available",
        count_word: "available",
        unavailable_age: age_or_max(unavail_from, now),
        degraded_age: age_or_max(degraded_from, now),
        rollout_age: None,
        unavailable_since: unavail_from.map(fmt),
        degraded_since: degraded_from.map(fmt),
    })
}

/// `first_seen_unsettled`: when Apex first saw this object not healthy
/// (tracked per uid, `triage::tracked`). StatefulSets carry no transition
/// times, so without it the clock falls back to creation.
pub fn statefulset_verdict(
    s: &StatefulSet,
    now: Timestamp,
    first_seen_unsettled: Option<Timestamp>,
) -> Verdict {
    let desired = s.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
    let st = s.status.as_ref();
    let ready = st.and_then(|s| s.ready_replicas).unwrap_or(0);
    let created_replicas = st.map(|s| s.replicas).unwrap_or(0);
    let rev_differs = st.is_some_and(
        |s| matches!((&s.current_revision, &s.update_revision), (Some(c), Some(u)) if c != u),
    );
    let generation = s.metadata.generation.unwrap_or(0);
    let observed = st.and_then(|s| s.observed_generation).unwrap_or(0);
    let (age, since) = unsettled_age(
        first_seen_unsettled,
        s.metadata.creation_timestamp.as_ref(),
        now,
    );
    replica_verdict(Replicas {
        desired,
        available: ready,
        rolling: rev_differs || observed < generation,
        scaling: created_replicas < desired,
        paused: false,
        ok_word: "Ready",
        count_word: "ready",
        unavailable_age: age,
        degraded_age: age,
        rollout_age: Some(age),
        unavailable_since: since.clone(),
        degraded_since: since,
    })
}

pub fn replicaset_verdict(
    r: &ReplicaSet,
    now: Timestamp,
    first_seen_unsettled: Option<Timestamp>,
) -> Verdict {
    let desired = r.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
    let st = r.status.as_ref();
    let ready = st.and_then(|s| s.ready_replicas).unwrap_or(0);
    let created_replicas = st.map(|s| s.replicas).unwrap_or(0);
    let (age, since) = unsettled_age(
        first_seen_unsettled,
        r.metadata.creation_timestamp.as_ref(),
        now,
    );
    let v = replica_verdict(Replicas {
        desired,
        available: ready,
        rolling: false,
        scaling: created_replicas < desired,
        paused: false,
        ok_word: "Ready",
        count_word: "ready",
        unavailable_age: age,
        degraded_age: age,
        rollout_age: Some(age),
        unavailable_since: since.clone(),
        degraded_since: since,
    });
    // A Deployment's ReplicaSet: the Deployment verdict owns the failure
    // (and Pulse counts it once, there), so cap the ReplicaSet at warn.
    let deployment_owned = r
        .metadata
        .owner_references
        .as_ref()
        .is_some_and(|o| o.iter().any(|o| o.kind == "Deployment"));
    if deployment_owned && v.is_fail() {
        return Verdict::warn(&v.rule_id, v.reason).since(v.since);
    }
    v
}

pub fn daemonset_verdict(
    ds: &DaemonSet,
    now: Timestamp,
    first_seen_unsettled: Option<Timestamp>,
) -> Verdict {
    let Some(st) = ds.status.as_ref() else {
        return Verdict::progressing("TRI-WL-005", "Starting");
    };
    let desired = st.desired_number_scheduled;
    if desired == 0 {
        return Verdict::neutral("TRI-WL-000", "No nodes");
    }
    let available = st.number_available.unwrap_or(0);
    let unavailable = st.number_unavailable.unwrap_or(0);
    let updated = st.updated_number_scheduled.unwrap_or(0);
    let generation = ds.metadata.generation.unwrap_or(0);
    let observed = st.observed_generation.unwrap_or(0);
    let (age, since) = unsettled_age(
        first_seen_unsettled,
        ds.metadata.creation_timestamp.as_ref(),
        now,
    );
    let label = format!("{available}/{desired} available");

    if available == 0 && age > WL_UNAVAILABLE_GRACE_S {
        return Verdict::fail("TRI-WL-004", Severity::Critical, label).since(since);
    }
    if st.number_misscheduled > 0 {
        return Verdict::warn("TRI-WL-004", "Misscheduled").since(since);
    }
    if observed < generation || updated < desired {
        if age > WL_ROLLOUT_DEADLINE_S {
            return Verdict::warn("TRI-WL-006", "Rollout stuck").since(since);
        }
        return Verdict::progressing("TRI-WL-005", "Rolling out");
    }
    if unavailable > 0 || available < desired {
        if age <= WL_DEGRADED_GRACE_S {
            return Verdict::progressing("TRI-WL-005", label);
        }
        return Verdict::warn("TRI-WL-004", label).since(since);
    }
    Verdict::ok("TRI-WL-000", "Available")
}

// ---------------------------------------------------------------- Jobs

pub fn job_verdict(job: &Job, _now: Timestamp) -> Verdict {
    let spec = job.spec.as_ref();
    let st = job.status.as_ref();
    let conds = st.and_then(|s| s.conditions.as_ref());
    let cond_true =
        |t: &str| conds.and_then(|c| c.iter().find(|c| c.type_ == t && c.status == "True"));

    if let Some(c) = cond_true("Failed") {
        let reason = c
            .reason
            .clone()
            .filter(|r| !r.is_empty())
            .unwrap_or_else(|| "Failed".into());
        return Verdict::fail("TRI-JOB-001", Severity::Error, reason)
            .since(c.last_transition_time.as_ref().map(fmt));
    }
    if cond_true("Complete").is_some() {
        return Verdict::ok("TRI-JOB-000", "Complete");
    }
    if spec.and_then(|s| s.suspend).unwrap_or(false) {
        return Verdict::neutral("TRI-JOB-000", "Suspended");
    }
    let active = st.and_then(|s| s.active).unwrap_or(0);
    let failed = st.and_then(|s| s.failed).unwrap_or(0);
    let succeeded = st.and_then(|s| s.succeeded).unwrap_or(0);
    let completions = spec.and_then(|s| s.completions).unwrap_or(1);
    if active > 0 && failed > 0 {
        return Verdict::warn("TRI-JOB-003", format!("{failed} failed, retrying"));
    }
    if active > 0 {
        return Verdict::progressing("TRI-JOB-000", "Running");
    }
    if succeeded >= completions {
        return Verdict::ok("TRI-JOB-000", "Complete");
    }
    Verdict::progressing("TRI-JOB-000", "Pending")
}

// ---------------------------------------------------------------- Nodes

pub fn node_verdict(node: &Node, _now: Timestamp) -> Verdict {
    let conds = node.status.as_ref().and_then(|s| s.conditions.as_ref());
    let cond = |t: &str| conds.and_then(|c| c.iter().find(|c| c.type_ == t));
    let since =
        |c: &k8s_openapi::api::core::v1::NodeCondition| c.last_transition_time.as_ref().map(fmt);

    let mut found = Vec::new();
    match cond("Ready") {
        Some(c) if c.status == "True" => {}
        Some(c) if c.status == "False" => found
            .push(Verdict::fail("TRI-NODE-001", Severity::Critical, "NotReady").since(since(c))),
        Some(c) => {
            found.push(Verdict::fail("TRI-NODE-001", Severity::Critical, "Unknown").since(since(c)))
        }
        None => found.push(Verdict::fail("TRI-NODE-001", Severity::Critical, "Unknown")),
    }
    if let Some(c) = cond("NetworkUnavailable").filter(|c| c.status == "True") {
        found.push(
            Verdict::fail("TRI-NODE-003", Severity::Critical, "NetworkUnavailable").since(since(c)),
        );
    }
    for p in ["MemoryPressure", "DiskPressure", "PIDPressure"] {
        if let Some(c) = cond(p).filter(|c| c.status == "True") {
            found.push(Verdict::fail("TRI-NODE-002", Severity::Error, p).since(since(c)));
        }
    }
    if node
        .spec
        .as_ref()
        .and_then(|s| s.unschedulable)
        .unwrap_or(false)
    {
        found.push(Verdict::warn("TRI-NODE-004", "Cordoned"));
    }
    found.push(Verdict::ok("TRI-NODE-000", "Ready"));
    worst(found).expect("non-empty")
}

// ---------------------------------------------------------------- PVCs

/// The scheduler sets this on a WaitForFirstConsumer claim once a pod using
/// it has been placed: from then on a pod is waiting on the volume.
pub const SELECTED_NODE_ANNOTATION: &str = "volume.kubernetes.io/selected-node";

/// `binding_mode`: the claim's StorageClass `volumeBindingMode`, when the
/// caller could look it up (None = unknown, e.g. no RBAC on StorageClasses).
pub fn pvc_verdict(
    pvc: &PersistentVolumeClaim,
    now: Timestamp,
    binding_mode: Option<&str>,
) -> Verdict {
    let phase = pvc
        .status
        .as_ref()
        .and_then(|s| s.phase.as_deref())
        .unwrap_or("Pending");
    let created = pvc.metadata.creation_timestamp.as_ref();
    match phase {
        "Bound" => Verdict::ok("TRI-PVC-000", "Bound"),
        "Lost" => Verdict::fail("TRI-PVC-002", Severity::Error, "Lost"),
        "Pending" => {
            let has_consumer = pvc
                .metadata
                .annotations
                .as_ref()
                .is_some_and(|a| a.contains_key(SELECTED_NODE_ANNOTATION));
            if binding_mode == Some("WaitForFirstConsumer") && !has_consumer {
                // Pending by design until a pod uses it: nothing is waiting.
                Verdict::neutral("TRI-PVC-003", "Waiting for consumer")
            } else if age_or_max(created, now) > PVC_PENDING_GRACE_S {
                Verdict::warn("TRI-PVC-001", "Pending").since(created.map(fmt))
            } else {
                Verdict::progressing("TRI-PVC-001", "Pending")
            }
        }
        other => Verdict::unknown("TRI-PVC-000", other),
    }
}
